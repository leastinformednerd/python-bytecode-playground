//! At some point I'd like to move to the technique described here
//! <https://purplesyringa.moe/blog/recovering-control-flow-structures-without-cfgs>
//! which is like, actually good

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use crate::bytecode::{
    defs::{Instr, StackItem},
    symbolic_evaluation::{AnnotatedBlock, BasicBlockToken, ConditionalJump, ControlFlowTag},
};

#[derive(Debug)]
pub enum PseudoASTTag {
    FallsThrough(BasicBlockToken),
    // breaks and continues having targets is purely for debugging convenience
    Breaks(BasicBlockToken),
    Continues(BasicBlockToken),
    WhileHead {
        jump: ConditionalJump,
        body: BasicBlockToken,
        falls_through_to: BasicBlockToken,
    },
    BareIf {
        jump: ConditionalJump,
        body: BasicBlockToken,
        falls_through_to: BasicBlockToken,
    },
    IfElse {
        jump: ConditionalJump,
        body: BasicBlockToken,
        r#else: BasicBlockToken,
        falls_through_to: BasicBlockToken,
    },
    ForLoop {
        body: BasicBlockToken,
        falls_through_to: BasicBlockToken,
        assignment: Instr,
    },
    Returns(StackItem),
    Passes,
}

#[derive(Debug)]
pub struct ResolvedBlock {
    pub body: Vec<Instr>,
    pub ast_tag: PseudoASTTag,
}

pub fn resolve_tags(
    resolving: BasicBlockToken,
    graph: &HashMap<BasicBlockToken, AnnotatedBlock>,
    out_map: &RefCell<HashMap<BasicBlockToken, ResolvedBlock>>,
) {
    if out_map.borrow().contains_key(&resolving) {
        return;
    }

    let AnnotatedBlock { cf_tag, body } = graph
        .get(&resolving)
        .expect("Basic block not in control flow graph");

    let tag = match cf_tag {
        ControlFlowTag::ForIter {
            assignment,
            found,
            exhausted,
        } => PseudoASTTag::ForLoop {
            body: *found,
            falls_through_to: *exhausted,
            assignment: assignment.clone(),
        },
        ControlFlowTag::JumpForward(to) => PseudoASTTag::Breaks(*to),
        ControlFlowTag::JumpBack(to) => PseudoASTTag::Continues(*to),
        ControlFlowTag::FallsThrough(to) => PseudoASTTag::FallsThrough(*to),
        ControlFlowTag::ConditionalJump {
            jump,
            met,
            otherwise,
        } => {
            if search_with_pred(
                *otherwise,
                |tok, graph| {
                    if let Some(AnnotatedBlock {
                        cf_tag: ControlFlowTag::JumpBack(to),
                        ..
                    }) = graph.get(&tok)
                        && *to == resolving
                    {
                        true
                    } else {
                        false
                    }
                },
                graph,
            )
            .is_some()
            {
                PseudoASTTag::WhileHead {
                    jump: jump.clone(),
                    body: *met,
                    falls_through_to: *otherwise,
                }
            } else if let Some(falls_through_to) = search_with_pred(
                *otherwise,
                |tok, graph| {
                    if let Some(AnnotatedBlock {
                        cf_tag: ControlFlowTag::JumpForward(jumps),
                        ..
                    }) = graph.get(&tok)
                        && search_with_pred(
                            *met,
                            |tok, graph| {
                                if let Some(AnnotatedBlock {
                                    cf_tag: ControlFlowTag::JumpForward(falls),
                                    ..
                                }) = graph.get(&tok)
                                    && falls == jumps
                                {
                                    true
                                } else {
                                    false
                                }
                            },
                            graph,
                        )
                        .is_some()
                    {
                        true
                    } else {
                        false
                    }
                },
                graph,
            ) {
                let mut guard = out_map.borrow_mut();
                let tok = if let Some(AnnotatedBlock {
                    cf_tag: ControlFlowTag::JumpForward(tok),
                    ..
                }) = graph.get(&falls_through_to)
                {
                    tok
                } else {
                    unreachable!()
                };

                let out = RefCell::new(HashSet::new());
                find_elses(*otherwise, *tok, graph, &out);
                find_elses(*met, *tok, graph, &out);
                for block in out.into_inner() {
                    guard.insert(
                        block,
                        ResolvedBlock {
                            body: graph[&block].body.clone(),
                            ast_tag: PseudoASTTag::Passes,
                        },
                    );
                }
                drop(guard);
                PseudoASTTag::IfElse {
                    jump: jump.clone(),
                    body: *otherwise,
                    r#else: *met,
                    falls_through_to,
                }
            } else {
                PseudoASTTag::BareIf {
                    jump: jump.clone(),
                    body: *otherwise,
                    falls_through_to: *met,
                }
            }
        }
        ControlFlowTag::Returns(val) => PseudoASTTag::Returns(val.clone()),
        ControlFlowTag::Dummy => panic!("Found a dummy val leaked into the cfg resolution"),
    };

    out_map.borrow_mut().insert(
        resolving,
        ResolvedBlock {
            body: body.clone(),
            ast_tag: tag,
        },
    );
}

// This really should be cached properly
fn terminates_with(
    start: BasicBlockToken,
    target: BasicBlockToken,
    graph: &HashMap<BasicBlockToken, AnnotatedBlock>,
) -> bool {
    fn cached(
        start: BasicBlockToken,
        target: BasicBlockToken,
        graph: &HashMap<BasicBlockToken, AnnotatedBlock>,
        seen: &RefCell<HashSet<BasicBlockToken>>,
    ) -> bool {
        if seen.borrow().contains(&start) {
            return false;
        }

        seen.borrow_mut().insert(start);

        if start == target {
            return true;
        }

        let AnnotatedBlock { cf_tag, .. } = graph
            .get(&start)
            .expect("Tried to find a node not in control flow graph");

        match cf_tag {
            ControlFlowTag::JumpForward(to) => cached(*to, target, graph, seen),
            ControlFlowTag::JumpBack(to) => cached(*to, target, graph, seen),
            ControlFlowTag::FallsThrough(to) => cached(*to, target, graph, seen),
            ControlFlowTag::ConditionalJump { met, otherwise, .. } => {
                if cached(*met, target, graph, seen) {
                    return true;
                }

                cached(*otherwise, target, graph, seen)
            }
            ControlFlowTag::ForIter {
                found, exhausted, ..
            } => {
                if cached(*found, target, graph, seen) {
                    return true;
                }

                cached(*exhausted, target, graph, seen)
            }
            ControlFlowTag::Returns(_) => false,
            ControlFlowTag::Dummy => unreachable!("Dummy leaked"),
        }
    }

    let seen = RefCell::new(HashSet::new());
    cached(start, target, graph, &seen)
}

fn search_with_pred(
    start: BasicBlockToken,
    pred: impl Fn(BasicBlockToken, &HashMap<BasicBlockToken, AnnotatedBlock>) -> bool,
    graph: &HashMap<BasicBlockToken, AnnotatedBlock>,
) -> Option<BasicBlockToken> {
    fn cached(
        start: BasicBlockToken,
        pred: &impl Fn(BasicBlockToken, &HashMap<BasicBlockToken, AnnotatedBlock>) -> bool,
        graph: &HashMap<BasicBlockToken, AnnotatedBlock>,
        seen: &RefCell<HashSet<BasicBlockToken>>,
    ) -> Option<BasicBlockToken> {
        if seen.borrow().contains(&start) {
            return None;
        }

        seen.borrow_mut().insert(start);

        if pred(start, graph) {
            return Some(start);
        }

        let AnnotatedBlock { cf_tag, .. } = graph
            .get(&start)
            .expect("Tried to find a node not in control flow graph");

        match cf_tag {
            ControlFlowTag::JumpForward(to) => cached(*to, pred, graph, seen),
            ControlFlowTag::JumpBack(to) => cached(*to, pred, graph, seen),
            ControlFlowTag::FallsThrough(to) => cached(*to, pred, graph, seen),
            ControlFlowTag::ConditionalJump { met, otherwise, .. } => {
                if let Some(token) = cached(*met, pred, graph, seen) {
                    return Some(token);
                }

                cached(*otherwise, pred, graph, seen)
            }
            ControlFlowTag::ForIter {
                found, exhausted, ..
            } => {
                if let Some(token) = cached(*found, pred, graph, seen) {
                    return Some(token);
                }

                cached(*exhausted, pred, graph, seen)
            }
            ControlFlowTag::Returns(_) => None,
            ControlFlowTag::Dummy => unreachable!("Dummy leaked"),
        }
    }

    let seen = RefCell::new(HashSet::new());
    cached(start, &pred, graph, &seen)
}

fn find_elses(
    start: BasicBlockToken,
    target: BasicBlockToken,
    graph: &HashMap<BasicBlockToken, AnnotatedBlock>,
    out: &RefCell<HashSet<BasicBlockToken>>,
) {
    fn cached(
        start: BasicBlockToken,
        target: BasicBlockToken,
        graph: &HashMap<BasicBlockToken, AnnotatedBlock>,
        seen: &RefCell<HashSet<BasicBlockToken>>,
        out: &RefCell<HashSet<BasicBlockToken>>,
    ) {
        if seen.borrow().contains(&start) {
            return;
        }

        println!("Trying to find elses at {start:?}");

        seen.borrow_mut().insert(start);

        let AnnotatedBlock { cf_tag, .. } = graph
            .get(&start)
            .expect("Tried to find a node not in control flow graph");

        match cf_tag {
            ControlFlowTag::JumpForward(to) | ControlFlowTag::FallsThrough(to) if *to == target => {
                println!("Found else at {start:?} to {to:?}");
                out.borrow_mut().insert(start);
            }
            ControlFlowTag::JumpForward(to) => cached(*to, target, graph, seen, out),
            ControlFlowTag::JumpBack(to) => cached(*to, target, graph, seen, out),
            ControlFlowTag::FallsThrough(to) => cached(*to, target, graph, seen, out),
            ControlFlowTag::ConditionalJump { met, otherwise, .. } => {
                cached(*met, target, graph, seen, out);
                cached(*otherwise, target, graph, seen, out);
            }
            ControlFlowTag::ForIter {
                found, exhausted, ..
            } => {
                cached(*found, target, graph, seen, out);
                cached(*exhausted, target, graph, seen, out);
            }
            ControlFlowTag::Returns(_) => {}
            ControlFlowTag::Dummy => unreachable!("Dummy leaked"),
        };
    }

    let seen = RefCell::new(HashSet::new());
    cached(start, target, graph, &seen, &out);
}
