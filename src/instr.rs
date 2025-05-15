use crate::heap::Reg;

pub enum Instruction {
    Move { dest: Reg, src: u32 },
    Add { arg0: Reg, arg1: Reg, ret: Reg },
}
