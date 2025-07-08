mod bytecode;

fn main() {
    // This is the instructions from example.py as a [u8]
    let test_code = [
        149, 0, 83, 0, 39, 0, 0, 0, 0, 0, 0, 0, 97, 14, 0, 0, 28, 0, 89, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        91, 2, 51, 1, 0, 0, 0, 0, 0, 0, 31, 0, 74, 21, 0, 0, 81, 0, 35, 0,
    ];

    bytecode::full_parse(&test_code);
}
