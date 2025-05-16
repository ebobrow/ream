use crate::{Reg, mem::DataObject};

pub enum Instruction {
    Move { dest: Reg, src: DataObject },
    Add { arg0: Reg, arg1: Reg, ret: Reg },
    Allocate { stack_need: usize },
}
