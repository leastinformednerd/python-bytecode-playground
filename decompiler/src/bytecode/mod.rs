use parse::parse;

pub mod defs;
mod parse;
mod symbolic_evaluation;

/// Performs the "full parsing" step, from a code object to the decompiler's
/// internal representation
pub fn full_parse(code: &[u8]) {
    let instrs = parse(code).unwrap();
    // debug_print_parse_instrs(&instrs);
    let res = symbolic_evaluation::eval_instructions(
        &instrs,
        &["x".into()],
        &["print".into()],
        &[std::rc::Rc::new(defs::PyConstInner::None)],
    );
    if let Ok(evaled) = res {
        println!("{evaled:#?}");
    } else {
        println!("Failed to parse right");
    }
}

// fn debug_print_parse_instrs(code: &[parse::ParseInstr]) {
//     for (index, instr) in code.iter().enumerate() {
//         println!("[{index}]: {:?} {}", instr.kind, instr.arg);
//     }
// }
