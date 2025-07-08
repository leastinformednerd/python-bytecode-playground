use std::collections::{BTreeSet, HashMap};
use std::fmt::Display;

use super::defs::{Instr, Name, PyConst, StackItem};
use super::parse::{ParseInstr, ParseInstrKind};

#[derive(Debug)]
struct BasicBlock<'a> {
    code: &'a [ParseInstr],
    children: BasicBlockChildren,
}

impl<'a> Display for BasicBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.children {
            BasicBlockChildren::Diverges => write!(f, "Block diverges"),
            BasicBlockChildren::FallsThroughTo(BasicBlockToken(n)) => write!(f, "Block->{n}"),
            BasicBlockChildren::CondJump {
                cond_met: BasicBlockToken(a),
                otherwise: BasicBlockToken(b),
            } => write!(f, "Block->(met: {a}, other: {b})"),
        }
    }
}

impl<'a> PartialEq for BasicBlock<'a> {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl<'a> Eq for BasicBlock<'a> {}

impl<'a> PartialOrd for BasicBlock<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}

impl<'a> Ord for BasicBlock<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
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
    FallsThroughTo(BasicBlockToken),
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
struct BasicBlockToken(usize);

type Stack = Vec<StackItem>;

pub struct Context<'a> {
    stack: Stack,
    locals: &'a [Name],
    consts: &'a [PyConst],
    globals: &'a [Name],
}

#[derive(Debug)]
pub enum SymbolicExecutionError {
    OutOfBoundsJump,
}

pub fn eval_instructions<'a>(
    instrs: &[ParseInstr],
    locals: &'a [Name],
    globals: &'a [Name],
    consts: &'a [PyConst],
) -> Result<Vec<Instr>, SymbolicExecutionError> {
    let block_map = create_blocks(instrs)?;

    todo!("END OF DEBUG");
}

fn create_blocks(
    instrs: &[ParseInstr],
) -> Result<HashMap<BasicBlockToken, BasicBlock>, SymbolicExecutionError> {
    // This is being used effectively as a read from once priority queue,
    // and could be BinaryHeap, but I think this should be faster, and I don't
    // want to profile a comparison
    let mut boundaries = BTreeSet::new();

    let mut jumps = Vec::new();

    for (index, instr) in instrs.into_iter().enumerate() {
        if let Some(delta) = instr.jump() {
            let jump_target = (index as isize + 1 + delta as isize) as usize;
            if jump_target >= instrs.len() {
                return Err(SymbolicExecutionError::OutOfBoundsJump);
            }

            jumps.push((
                index,
                (
                    jump_target,
                    if instr.cond_jump() {
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
                    code: instrs,
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
    // The final block ends at the end of the program code
    boundaries.insert(instrs.len() - 1);
    // let mut boundaries = boundaries
    //     .into_iter()
    //     .chain(std::iter::once(instrs.len() - 1));
    let mut boundaries = boundaries.into_iter();
    for boundary in &mut boundaries {
        let children;
        if prev <= jump_cache && jump_cache < boundary {
            children = match to {
                (a, b) if a == b && prev <= a && a < boundary => BasicBlockChildren::Diverges,
                (a, b) if a == b => BasicBlockChildren::FallsThroughTo(BasicBlockToken(a)),
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
                            code: &instrs[prev..boundary],
                            children,
                        },
                    );
                    prev = boundary;
                    break;
                }
            }
        } else {
            children = BasicBlockChildren::Diverges;
        };

        block_map.insert(
            BasicBlockToken(prev),
            BasicBlock {
                code: &instrs[prev..boundary],
                children,
            },
        );

        prev = boundary;
    }

    for boundary in boundaries {
        block_map.insert(
            BasicBlockToken(boundary),
            BasicBlock {
                code: &instrs[prev..boundary],
                children: BasicBlockChildren::Diverges,
            },
        );
        prev = boundary;
    }

    Ok(block_map)
}
