use crate::{Reg, mem::DataObject};

pub enum Instruction {
    Move { dest: Reg, src: DataObject },
    Add { arg0: Reg, arg1: Reg, ret: Reg },
    Allocate { stack_need: usize },

    IsLt { lbl: usize, arg0: Reg, arg1: Reg },
    IsGe { lbl: usize, arg0: Reg, arg1: Reg },
    IsEq { lbl: usize, arg0: Reg, arg1: Reg },
    IsNe { lbl: usize, arg0: Reg, arg1: Reg },

    IsInteger { lbl: usize, arg: Reg },

    Ret,
    Call { ip: usize },
}
