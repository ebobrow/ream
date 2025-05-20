use crate::{Reg, mem::DataObject};

// TODO: wish we didn't have to clone the dataobject
#[derive(Debug, Clone)]
pub enum Instruction {
    Move { dest: Reg, src: DataObject },
    Add { arg0: Reg, arg1: Reg, ret: Reg },
    Allocate { stack_need: usize },

    // TODO: actual labels and not offsets
    IsLt { lbl: usize, arg0: Reg, arg1: Reg },
    IsGe { lbl: usize, arg0: Reg, arg1: Reg },
    IsEq { lbl: usize, arg0: Reg, arg1: Reg },
    IsNe { lbl: usize, arg0: Reg, arg1: Reg },

    IsInteger { lbl: usize, arg: Reg },

    Jmp { lbl: usize },
    Ret,
    Call { ip: usize },

    // TODO: is this how we do this
    Spawn { instrs: Vec<Instruction> },

    Send,
    Wait,
}
