use crate::{Item, List, PID, Reg, mem::DataObject};

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

impl From<&List> for Instruction {
    fn from(list: &List) -> Self {
        if let Some(Item::Atom(instr)) = list.first() {
            match &instr[..] {
                "move" => {
                    assert_eq!(list.len(), 3);
                    let dest = Reg::from(list[1].expect_list());
                    let src = DataObject::from(&list[2]);
                    Instruction::Move { dest, src }
                }
                "add" => {
                    assert_eq!(list.len(), 4);
                    let arg0 = Reg::from(list[1].expect_list());
                    let arg1 = Reg::from(list[2].expect_list());
                    let ret = Reg::from(list[3].expect_list());
                    Instruction::Add { arg0, arg1, ret }
                }
                "alloc" => {
                    assert_eq!(list.len(), 2);
                    let stack_need = list[1].expect_num().try_into().unwrap();
                    Instruction::Allocate { stack_need }
                }
                "is_lt" => {
                    assert_eq!(list.len(), 4);
                    let lbl = list[1].expect_num().try_into().unwrap();
                    let arg0 = Reg::from(list[2].expect_list());
                    let arg1 = Reg::from(list[3].expect_list());
                    Instruction::IsLt { lbl, arg0, arg1 }
                }
                "is_ge" => {
                    assert_eq!(list.len(), 4);
                    let lbl = list[1].expect_num().try_into().unwrap();
                    let arg0 = Reg::from(list[2].expect_list());
                    let arg1 = Reg::from(list[3].expect_list());
                    Instruction::IsGe { lbl, arg0, arg1 }
                }
                "is_eq" => {
                    assert_eq!(list.len(), 4);
                    let lbl = list[1].expect_num().try_into().unwrap();
                    let arg0 = Reg::from(list[2].expect_list());
                    let arg1 = Reg::from(list[3].expect_list());
                    Instruction::IsEq { lbl, arg0, arg1 }
                }
                "is_ne" => {
                    assert_eq!(list.len(), 4);
                    let lbl = list[1].expect_num().try_into().unwrap();
                    let arg0 = Reg::from(list[2].expect_list());
                    let arg1 = Reg::from(list[3].expect_list());
                    Instruction::IsNe { lbl, arg0, arg1 }
                }
                "is_int" => {
                    assert_eq!(list.len(), 3);
                    let lbl = list[1].expect_num().try_into().unwrap();
                    let arg = Reg::from(list[2].expect_list());
                    Instruction::IsInteger { lbl, arg }
                }
                "jmp" => {
                    assert_eq!(list.len(), 2);
                    let lbl = list[1].expect_num().try_into().unwrap();
                    Instruction::Jmp { lbl }
                }
                "ret" => {
                    assert_eq!(list.len(), 1);
                    Instruction::Ret
                }
                "call" => {
                    assert_eq!(list.len(), 2);
                    let ip = list[1].expect_num().try_into().unwrap();
                    Instruction::Call { ip }
                }
                "spawn" => {
                    assert_eq!(list.len(), 2);
                    let instrs = list[1]
                        .expect_list()
                        .iter()
                        .map(|l| Instruction::from(l.expect_list()))
                        .collect();
                    Instruction::Spawn { instrs }
                }
                "send" => {
                    assert_eq!(list.len(), 1);
                    Instruction::Send
                }
                "wait" => {
                    assert_eq!(list.len(), 1);
                    Instruction::Wait
                }
                _ => panic!("unknown instruction {instr}"),
            }
        } else {
            panic!("invalid instruction");
        }
    }
}

impl From<&List> for Reg {
    fn from(list: &List) -> Self {
        if let Some(Item::Atom(instr)) = list.first() {
            match &instr[..] {
                "x" => {
                    assert_eq!(list.len(), 2);
                    if let Item::Num(x) = list[1] {
                        Reg::X(x.try_into().unwrap())
                    } else {
                        panic!("invalid register");
                    }
                }
                "y" => {
                    assert_eq!(list.len(), 2);
                    if let Item::Num(x) = list[1] {
                        Reg::Y(x.try_into().unwrap())
                    } else {
                        panic!("invalid register");
                    }
                }
                "Htop" => {
                    assert_eq!(list.len(), 1);
                    Reg::Htop
                }
                "E" => {
                    assert_eq!(list.len(), 1);
                    Reg::E
                }
                "I" => {
                    assert_eq!(list.len(), 1);
                    Reg::I
                }
                "FP" => {
                    assert_eq!(list.len(), 1);
                    Reg::FP
                }
                "CP" => {
                    assert_eq!(list.len(), 1);
                    Reg::CP
                }
                "fcalls" => {
                    assert_eq!(list.len(), 1);
                    Reg::fcalls
                }
                _ => panic!("unknown instruction {instr}"),
            }
        } else {
            panic!("invalid register");
        }
    }
}

impl From<&Item> for DataObject {
    fn from(value: &Item) -> Self {
        match value {
            Item::Num(x) => DataObject::Small(*x),
            Item::Atom(x) => DataObject::Atom(x.clone()),
            Item::List(x) => match x.first().unwrap().expect_atom() {
                "nil" => {
                    assert_eq!(x.len(), 1);
                    DataObject::Nil
                }
                "pid" => {
                    assert_eq!(x.len(), 3);
                    DataObject::Pid(PID::new(
                        x[1].expect_num().try_into().unwrap(),
                        x[2].expect_num().try_into().unwrap(),
                    ))
                }
                _ => todo!(),
            },
        }
    }
}
