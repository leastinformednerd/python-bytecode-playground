use bytecode::defs::PyConstInner;

mod bytecode;
mod codegen;

fn main() {
    // This is the instructions from example.py:f as a [u8]
    let test_code = [
        149, 0, 83, 0, 39, 0, 0, 0, 0, 0, 0, 0, 97, 38, 0, 0, 28, 0, 83, 0, 91, 2, 44, 5, 0, 0, 39,
        0, 0, 0, 0, 0, 0, 0, 97, 14, 0, 0, 28, 0, 89, 1, 0, 0, 0, 0, 0, 0, 0, 0, 91, 2, 51, 1, 0,
        0, 0, 0, 0, 0, 31, 0, 74, 32, 0, 0, 89, 1, 0, 0, 0, 0, 0, 0, 0, 0, 91, 3, 51, 1, 0, 0, 0,
        0, 0, 0, 31, 0, 74, 45, 0, 0, 81, 0, 35, 0,
    ];

    let parsed = bytecode::parse(&test_code).unwrap();

    let parsed = bytecode::eval_instructions(
        &parsed,
        &["x".into()],
        &["print".into()],
        &[std::rc::Rc::new(bytecode::defs::PyConstInner::None)],
    )
    .expect("For testing I assume this succeeds");

    println!("Test 1:");
    codegen::gen_code(&parsed, std::io::stdout());
    print!("\n");

    // This is the instructions from example.py:g as a [u8]
    let test2 = [
        149, 0, 89, 1, 0, 0, 0, 0, 0, 0, 0, 0, 83, 0, 51, 1, 0, 0, 0, 0, 0, 0, 16, 0, 69, 27, 0, 0,
        109, 1, 83, 1, 91, 5, 44, 10, 0, 0, 91, 0, 56, 88, 0, 0, 97, 3, 0, 0, 28, 0, 74, 15, 0, 0,
        83, 1, 91, 7, 44, 10, 0, 0, 91, 8, 56, 88, 0, 0, 100, 3, 0, 0, 28, 0, 74, 27, 0, 0, 31, 0,
        76, 2, 9, 0, 30, 0, 85, 1, 91, 11, 44, 10, 0, 0, 91, 9, 56, 88, 0, 0, 97, 3, 0, 0, 28, 0,
        91, 5, 35, 0, 81, 0, 35, 0,
    ];

    let parsed = bytecode::parse(&test2).unwrap();

    let parsed = bytecode::eval_instructions(
        &parsed,
        &["x".into(), "i".into()],
        &["range".into()],
        &[std::rc::Rc::new(bytecode::defs::PyConstInner::None)],
    )
    .expect("for testing expecting parsing to work");

    println!("Test 2");
    codegen::gen_code(&parsed, std::io::stdout());
    print!("\n");

    // This is the instructions from example.py:h as a [u8]
    let test3 = [
        149, 0, 89, 1, 0, 0, 0, 0, 0, 0, 0, 0, 83, 0, 51, 1, 0, 0, 0, 0, 0, 0, 16, 0, 69, 72, 0, 0,
        109, 1, 83, 0, 91, 2, 56, 18, 0, 0, 100, 3, 0, 0, 28, 0, 74, 12, 0, 0, 83, 0, 91, 5, 56,
        148, 0, 0, 97, 13, 0, 0, 28, 0, 89, 3, 0, 0, 0, 0, 0, 0, 0, 0, 83, 0, 51, 1, 0, 0, 0, 0, 0,
        0, 31, 0, 76, 30, 83, 0, 91, 3, 56, 18, 0, 0, 97, 13, 0, 0, 28, 0, 89, 3, 0, 0, 0, 0, 0, 0,
        0, 0, 83, 0, 51, 1, 0, 0, 0, 0, 0, 0, 31, 0, 76, 11, 89, 3, 0, 0, 0, 0, 0, 0, 0, 0, 83, 0,
        51, 1, 0, 0, 0, 0, 0, 0, 31, 0, 89, 3, 0, 0, 0, 0, 0, 0, 0, 0, 83, 0, 51, 1, 0, 0, 0, 0, 0,
        0, 31, 0, 74, 71, 0, 0, 9, 0, 30, 0, 81, 0, 35, 0,
    ];

    let parsed = bytecode::parse(&test3).unwrap();

    let parsed = bytecode::eval_instructions(
        &parsed,
        &["x".into(), "i".into()],
        &["range".into(), "print".into()],
        &[std::rc::Rc::new(bytecode::defs::PyConstInner::None)],
    )
    .expect("for testing expecting parsing to work");

    println!("Test 3");
    codegen::gen_code(&parsed, std::io::stdout());
    print!("\n");

    let test4 = [
        149, 0, 89, 1, 0, 0, 0, 0, 0, 0, 0, 0, 86, 18, 56, 148, 0, 0, 97, 9, 0, 0, 28, 0, 83, 0,
        51, 1, 0, 0, 0, 0, 0, 0, 31, 0, 81, 0, 35, 0, 83, 3, 51, 1, 0, 0, 0, 0, 0, 0, 31, 0, 81, 0,
        35, 0,
    ];

    let parsed = bytecode::parse(&test4).unwrap();

    let parsed = bytecode::eval_instructions(
        &parsed,
        &["a".into(), "b".into(), "c".into(), "d".into()],
        &["print".into()],
        &[std::rc::Rc::new(bytecode::defs::PyConstInner::None)],
    )
    .expect("for testing assume this parses");

    println!("Test 4");
    codegen::gen_code(&parsed, std::io::stdout());
    print!("\n");

    let test5 = [
        149, 0, 89, 1, 0, 0, 0, 0, 0, 0, 0, 0, 83, 0, 51, 1, 0, 0, 0, 0, 0, 0, 109, 1, 91, 0, 109,
        2, 89, 3, 0, 0, 0, 0, 0, 0, 0, 0, 83, 1, 51, 1, 0, 0, 0, 0, 0, 0, 16, 0, 69, 7, 0, 0, 109,
        3, 86, 35, 44, 13, 0, 0, 109, 2, 74, 9, 0, 0, 9, 0, 30, 0, 86, 33, 83, 1, 91, 1, 44, 10, 0,
        0, 44, 5, 0, 0, 91, 2, 44, 2, 0, 0, 56, 88, 0, 0, 97, 14, 0, 0, 28, 0, 89, 5, 0, 0, 0, 0,
        0, 0, 0, 0, 81, 0, 51, 1, 0, 0, 0, 0, 0, 0, 31, 0, 81, 2, 35, 0, 89, 5, 0, 0, 0, 0, 0, 0,
        0, 0, 81, 1, 51, 1, 0, 0, 0, 0, 0, 0, 31, 0, 81, 2, 35, 0,
    ];

    let parsed = bytecode::parse(&test5).unwrap();

    let parsed = bytecode::eval_instructions(
        &parsed,
        &["x".into(), "y".into(), "acc".into(), "i".into()],
        &["int".into(), "range".into(), "print".into()],
        &[
            std::rc::Rc::new(bytecode::defs::PyConstInner::StringLiteral(
                "Correctly found that sum of 0..x = x*(x+1)/2".into(),
            )),
            std::rc::Rc::new(PyConstInner::StringLiteral(
                "Incorrecly analysed the sum of 0..x".into(),
            )),
            std::rc::Rc::new(PyConstInner::None),
        ],
    )
    .expect("for testing assume this parses");

    println!("Test 5");
    codegen::gen_code(&parsed, std::io::stdout());
    print!("\n");
}
