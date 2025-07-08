use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Display;

use super::defs::{Instr, Name, PyConst, PyConstInner, StackItem};
use super::parse::{ParseInstr, ParseInstrKind};

macro_rules! pop_into {
    ($ctx:ident, $($to:ident),*) => {
        $(let $to = $ctx
            .stack
            .pop()
            .ok_or(SymbolicEvaluationError::MissingStackItem)?;)*
    };
}

#[derive(Debug)]
struct BasicBlock {
    at: BasicBlockToken,
    code: Vec<ParseInstr>,
    children: BasicBlockChildren,
}

impl BasicBlock {
    fn get1(&self) -> Result<BasicBlockToken, SymbolicEvaluationError> {
        match self.children {
            BasicBlockChildren::LeadsTo(token) => Ok(token),
            _ => Err(SymbolicEvaluationError::WrongBlockChildCount),
        }
    }
    fn get2(&self) -> Result<(BasicBlockToken, BasicBlockToken), SymbolicEvaluationError> {
        match self.children {
            BasicBlockChildren::CondJump {
                cond_met,
                otherwise,
            } => Ok((cond_met, otherwise)),
            _ => Err(SymbolicEvaluationError::WrongBlockChildCount),
        }
    }
    fn get0(&self) -> Result<(), SymbolicEvaluationError> {
        match self.children {
            BasicBlockChildren::Diverges => Ok(()),
            _ => Err(SymbolicEvaluationError::WrongBlockChildCount),
        }
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.children {
            BasicBlockChildren::Diverges => write!(f, "Block diverges"),
            BasicBlockChildren::LeadsTo(BasicBlockToken(n)) => write!(f, "Block->{n}"),
            BasicBlockChildren::CondJump {
                cond_met: BasicBlockToken(a),
                otherwise: BasicBlockToken(b),
            } => write!(f, "Block->(met: {a}, other: {b})"),
        }
    }
}

impl PartialEq for BasicBlock {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for BasicBlock {}

impl PartialOrd for BasicBlock {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}

impl Ord for BasicBlock {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

/// This represents the relations between basic blocks in terms of what other
/// blocks are directly reachable from this one
#[derive(Debug)]
enum BasicBlockChildren {
    /// Represents every conditional jump, such as those produced by loops
    /// or more directly by conditionals like if / if ... else
    CondJump {
        cond_met: BasicBlockToken,
        otherwise: BasicBlockToken,
    },
    /// This is all the "falling through" operations, whether in the bytecode
    /// or implied in the block following an if
    LeadsTo(BasicBlockToken),
    /// The block either enters an infinite loop or returns, and as such can't
    /// lead to another basic block in the same function (the execution unit
    /// in CPython)
    Diverges,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
// BasicBlockTokens are indices into a HashMap to BasicBlocks,
// and act as shared references to the basic block referred to
//
// They wrap a usize which is the index into the instruction array that they
// point to, meaning it's trivial to precreate these before a block is actually
// made. This does violate a little the proof like property of them, but if the
// implementation of block seperation is correct, then a basic block is
// guaranteed to start at that point.
pub struct BasicBlockToken(usize);

// This is just so I can what I use to refer to blocks externally to this
// module as easily
pub type BlockToken = BasicBlockToken;

type Stack = Vec<StackItem>;

#[derive(Clone)]
pub struct Context<'a> {
    stack: Stack,
    locals: &'a [Name],
    consts: &'a [PyConst],
    globals: &'a [Name],
    block_map: &'a HashMap<BasicBlockToken, BasicBlock>,
    out_map: &'a RefCell<HashMap<BasicBlockToken, Vec<Instr>>>,
}

#[derive(Debug)]
pub enum SymbolicEvaluationError {
    OutOfBoundsJump,
    MissingStackItem,
    InvalidOperationTag,
    WrongBlockChildCount,
}

/// Eval instructions takes the necessary parts of a code object and returns a
/// series of blocks that makes up that code object, along with the computational
/// effects that take place within each of those blocks
pub fn eval_instructions<'a>(
    instrs: &[ParseInstr],
    locals: &'a [Name],
    globals: &'a [Name],
    consts: &'a [PyConst],
) -> Result<HashMap<BasicBlockToken, Vec<Instr>>, SymbolicEvaluationError> {
    let block_map = create_blocks(instrs)?;
    let out_map = RefCell::new(HashMap::new());
    let ctx = Context {
        stack: Stack::new(),
        block_map: &block_map,
        out_map: &out_map,
        locals,
        globals,
        consts,
    };

    eval_block(&block_map[&BasicBlockToken(0)], ctx)?;

    // This is hacky, but it can't fail
    // .take() also shouldn't fail but there's no take() variant that returns
    // an Option, it just panics without letting the user log anything
    Ok(out_map.replace(HashMap::new()))
}

fn eval_block<'a>(block: &BasicBlock, mut ctx: Context<'a>) -> Result<(), SymbolicEvaluationError> {
    if ctx.out_map.borrow().contains_key(&block.at) {
        return Ok(());
    }
    use ParseInstr as I;
    use ParseInstrKind as K;
    use StackItem as S;

    let mut acc = Vec::new();
    for (index, instr) in block.code.iter().enumerate() {
        match instr {
            instr if instr.is_terminal() => {
                if index == block.code.len() - 1 {
                    break;
                } else {
                    panic!("Found a terminal that wasn't at the end of a block");
                }
            }
            I {
                kind: K::LoadConst,
                arg,
            } => ctx.stack.push(S::Const(ctx.consts[*arg as usize].clone())),
            I {
                kind: K::LoadGlobal,
                arg,
            } => {
                if arg & 1 == 1 {
                    ctx.stack.push(S::Null)
                };
                ctx.stack
                    .push(S::Global(ctx.globals[*arg as usize >> 1].clone()))
            }
            I {
                kind: K::LoadSmallInt,
                arg,
            } => ctx
                .stack
                .push(S::Const(std::rc::Rc::new(PyConstInner::Int(*arg as i64)))),
            I {
                kind: K::LoadFast,
                arg,
            } => {
                if arg & 1 == 1 {
                    ctx.stack.push(S::Null)
                };
                ctx.stack
                    .push(S::Global(ctx.locals[*arg as usize >> 1].clone()))
            }
            I {
                kind: K::StoreFast,
                arg,
            } => {
                pop_into!(ctx, top);
                acc.push(Instr::StoreFast(ctx.locals[*arg as usize].clone(), top));
            }
            I {
                kind: K::StoreGlobal,
                arg,
            } => {
                pop_into!(ctx, top);
                acc.push(Instr::StoreGlobal(ctx.globals[*arg as usize].clone(), top));
            }
            I {
                kind: K::EndFor | K::PopTop | K::PopIter,
                ..
            } => {
                ctx.stack
                    .pop()
                    .ok_or(SymbolicEvaluationError::MissingStackItem)?;
            }
            I {
                kind: K::GetIter, ..
            } => {
                pop_into!(ctx, top);
                ctx.stack
                    .push(StackItem::Derived(Box::new(Instr::GetIter(top))));
            }
            I {
                kind: K::BinaryOp,
                arg,
            } => {
                pop_into!(ctx, rhs, lhs);
                ctx.stack.push(StackItem::Derived(Box::new(Instr::BinaryOp(
                    super::defs::BinaryOp::try_from((*arg & 255) as u8)
                        .or(Err(SymbolicEvaluationError::InvalidOperationTag))?,
                    lhs,
                    rhs,
                ))))
            }
            I {
                kind: K::CompareOp,
                arg,
            } => {
                pop_into!(ctx, rhs, lhs);
                ctx.stack.push(StackItem::Derived(Box::new(Instr::CompareOp(
                    super::defs::ComparisonOp::try_from((*arg & 255) as u8)
                        .or(Err(SymbolicEvaluationError::InvalidOperationTag))?,
                    lhs,
                    rhs,
                ))))
            }
            I {
                kind: K::MakeFunction,
                ..
            } => {
                pop_into!(ctx, f);
                let ok = if let StackItem::Const(rc_inner) = f {
                    if let PyConstInner::CodeObject(..) = *rc_inner {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if !ok {
                    // This is unlikely to change, I don't think there's any
                    // way to get the python compiler emit any code other than
                    // LOAD_CONST, MAKE_FUNCTION
                    panic!(
                        "Currently MAKE_FUNCTION run on any symbol other than a constant code object is unsupported"
                    );
                }
            }
            I {
                kind: K::ToBool, ..
            } => {
                pop_into!(ctx, top);
                ctx.stack
                    .push(StackItem::Derived(Box::new(Instr::ToBool(top))));
            }
            I { kind: K::Call, arg } => {
                assert!(*arg >= 0);
                if ctx.stack.len() < (2 + *arg) as usize {
                    return Err(SymbolicEvaluationError::MissingStackItem);
                }
                let args = ctx.stack.split_off(ctx.stack.len() - (*arg as usize));
                pop_into!(ctx, meth, obj);
                let instr = Instr::Call { obj, meth, args };
                ctx.stack.push(StackItem::Derived(Box::new(instr.clone())));
                acc.push(instr);
            }
            I {
                kind: K::Resume, ..
            } => {}
            instr if instr.is_nop() => {}
            _ => unreachable!(),
        };
    }

    if let Some(terminal) = block.code.last()
        && terminal.is_terminal()
    {
        let instr = match terminal {
            I {
                kind: K::PopJumpIfTrue,
                ..
            } => {
                pop_into!(ctx, cond);
                let (met, otherwise) = block.get2()?;
                Instr::PopJumpIfTrue {
                    cond,
                    met,
                    otherwise,
                }
            }
            I {
                kind: K::PopJumpIfFalse,
                ..
            } => {
                pop_into!(ctx, cond);
                let (met, otherwise) = block.get2()?;
                Instr::PopJumpIfFalse {
                    cond,
                    met,
                    otherwise,
                }
            }
            I {
                kind: K::PopJumpIfNone,
                ..
            } => {
                pop_into!(ctx, cond);
                let (met, otherwise) = block.get2()?;
                Instr::PopJumpIfNone {
                    cond,
                    met,
                    otherwise,
                }
            }
            I {
                kind: K::PopJumpIfNotNone,
                ..
            } => {
                pop_into!(ctx, cond);
                let (met, otherwise) = block.get2()?;
                Instr::PopJumpIfNotNone {
                    cond,
                    met,
                    otherwise,
                }
            }
            I {
                kind: K::ForIter, ..
            } => {
                pop_into!(ctx, cond);
                let (found_val, exhausted) = block.get2()?;
                Instr::ForIter {
                    cond,
                    found_val,
                    exhausted,
                }
            }
            I {
                kind: K::JumpBackward,
                ..
            } => Instr::JumpBackward(block.get1()?),
            I {
                kind: K::JumpForward,
                ..
            } => Instr::JumpForward(block.get1()?),
            I {
                kind: K::ReturnValue,
                ..
            } => {
                pop_into!(ctx, ret);
                block.get0()?;
                Instr::ReturnValue(ret)
            }
            _ => unreachable!(),
        };
        acc.push(instr);
    }

    ctx.out_map.borrow_mut().insert(block.at, acc);

    match block.children {
        BasicBlockChildren::CondJump {
            cond_met,
            otherwise,
        } => {
            eval_block(&ctx.block_map[&cond_met], ctx.clone())?;
            eval_block(&ctx.block_map[&otherwise], ctx)?;
        }
        BasicBlockChildren::LeadsTo(block) => {
            eval_block(&ctx.block_map[&block], ctx)?;
        }
        BasicBlockChildren::Diverges => {}
    }

    Ok(())
}

fn create_blocks(
    instrs: &[ParseInstr],
) -> Result<HashMap<BasicBlockToken, BasicBlock>, SymbolicEvaluationError> {
    // This is being used effectively as a read from once priority queue,
    // and could be BinaryHeap, but I think this should be faster, and I don't
    // want to profile a comparison
    let mut boundaries = BTreeSet::new();

    let mut jumps = Vec::new();

    for (index, instr) in instrs.into_iter().enumerate() {
        if let Some(delta) = instr.jump() {
            let jump_target = (index as isize + delta as isize) as usize;
            if jump_target >= instrs.len() {
                return Err(SymbolicEvaluationError::OutOfBoundsJump);
            }

            jumps.push((
                index,
                (
                    jump_target,
                    if instr.is_cond_jump() {
                        index + 1
                    } else {
                        jump_target
                    },
                ),
            ));

            boundaries.insert(jump_target);

            boundaries.insert(index + 1);
        }
    }

    let mut block_map = HashMap::new();
    let mut cur_jump_index = 0;
    let (mut jump_cache, mut to) = match jumps.get(0) {
        Some(n) => *n,
        None => {
            block_map.insert(
                BasicBlockToken(0),
                BasicBlock {
                    at: BasicBlockToken(0),
                    code: remove_nops(instrs),
                    children: BasicBlockChildren::Diverges,
                },
            );
            return Ok(block_map);
        }
    };

    let mut prev = 0;

    // prev being 0 is semantically the boundary at the start of the root block
    // so it's being "moved" out of it

    boundaries.remove(&0);
    // The last block ends at the end of the instructions
    boundaries.insert(instrs.len());

    let mut boundaries = boundaries.into_iter();
    for boundary in &mut boundaries {
        let children;
        if prev <= jump_cache && jump_cache < boundary {
            children = match to {
                (a, b) if a == b && prev <= a && a < boundary => BasicBlockChildren::Diverges,
                (a, b) if a == b => BasicBlockChildren::LeadsTo(BasicBlockToken(a)),
                (a, b) => BasicBlockChildren::CondJump {
                    cond_met: BasicBlockToken(a),
                    otherwise: BasicBlockToken(b),
                },
            };
            cur_jump_index += 1;
            (jump_cache, to) = match jumps.get(cur_jump_index) {
                Some(n) => *n,
                None => {
                    block_map.insert(
                        BasicBlockToken(prev),
                        BasicBlock {
                            at: BasicBlockToken(prev),
                            code: remove_nops(&instrs[prev..boundary]),
                            children,
                        },
                    );
                    prev = boundary;
                    break;
                }
            }
        } else {
            children = match instrs[boundary - 1] {
                ParseInstr {
                    kind: ParseInstrKind::ReturnValue,
                    ..
                } => BasicBlockChildren::Diverges,
                instr if instr.jump().is_some() => {
                    panic!("A jump instruction leaked through the jump pass");
                }

                _ => BasicBlockChildren::LeadsTo(BasicBlockToken(boundary)),
            };
        };

        block_map.insert(
            BasicBlockToken(prev),
            BasicBlock {
                at: BasicBlockToken(prev),
                code: remove_nops(&instrs[prev..boundary]),
                children,
            },
        );

        prev = boundary;
    }

    for boundary in boundaries {
        println!("instr={:#?}", instrs[boundary - 1]);
        let children = match instrs[boundary - 1] {
            ParseInstr {
                kind: ParseInstrKind::ReturnValue,
                ..
            } => BasicBlockChildren::Diverges,
            instr if instr.jump().is_some() => {
                panic!("A jump instruction leaked through the jump pass");
            }

            _ => BasicBlockChildren::LeadsTo(BasicBlockToken(boundary)),
        };
        println!("children = {children:#?}");

        block_map.insert(
            BasicBlockToken(prev),
            BasicBlock {
                at: BasicBlockToken(prev),
                code: remove_nops(&instrs[prev..boundary]),
                children,
            },
        );
        prev = boundary;
    }

    Ok(block_map)
}

fn remove_nops(code: &[ParseInstr]) -> Vec<ParseInstr> {
    code.into_iter()
        .filter_map(|instr| if !instr.is_nop() { Some(*instr) } else { None })
        .collect::<Vec<_>>()
}
