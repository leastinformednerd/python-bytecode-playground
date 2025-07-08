use parse::parse;

pub mod defs;
mod parse;
mod symbolic_evaluation;

/// Performs the "full parsing" step, from a code object to the decompiler's
/// internal representation
pub fn full_parse(code: &[u8]) {
    let instrs = parse(code).unwrap();
    let _ = symbolic_evaluation::eval_instructions(&instrs, &[], &[], &[]);
}
