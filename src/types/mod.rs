mod data_holder;
mod instruction;
mod jumps;
mod operation;

pub use data_holder::*;
pub use instruction::*;
pub use jumps::*;
pub use operation::*;

pub type Line<'a> = (Option<&'a str>, Instruction<'a>);

#[derive(Debug)]
pub struct Error(pub String);
