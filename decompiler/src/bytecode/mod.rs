use parse::parse;

pub mod defs;
mod parse;
mod symbolic_evaluation;

/// Performs the "full parsing" step, from a code object to the decompiler's
/// internal representation
pub fn full_parse(code: &[u8]) {
    let instrs = parse(code).unwrap();
    debug_print_parse_instrs(&instrs);
    let _ = symbolic_evaluation::eval_instructions(&instrs, &[], &[], &[]);
}

fn debug_print_parse_instrs(code: &[parse::ParseInstr]) {
    for (index, instr) in code.iter().enumerate() {
        println!("[{index}]: {:?} {}", instr.kind, instr.arg);
    }
}
