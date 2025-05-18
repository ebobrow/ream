#![feature(mapped_lock_guards)]

pub use instr::Instruction;
pub use mem::{DataObject, stack::Reg};
pub use vm::VM;

mod instr;
mod mem;
mod pcb;
mod scheduler;
mod vm;
