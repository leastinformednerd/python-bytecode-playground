pub mod defs;
pub mod parse;
pub mod symbolic_evaluation;

pub use parse::parse;
pub use symbolic_evaluation::eval_instructions;
