use crate::{DataObject, Instruction, PID, Reg};

lrlex::lrlex_mod!("byte.l");
lrpar::lrpar_mod!("byte.y");

pub use byte_y::{Item, List, Prog};

type Label = (usize, usize);
fn get_label(labels: &[Label], item: &Item) -> usize {
    let n = item.expect_num();
    labels.iter().find(|(name, _)| name == &n).unwrap().1
}
impl From<(&[Label], &List)> for Instruction {
    fn from(value: (&[Label], &Vec<Item>)) -> Self {
        let (labels, list) = value;
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
                    let stack_need = list[1].expect_num();
                    Instruction::Allocate { stack_need }
                }
                "is_lt" => {
                    assert_eq!(list.len(), 4);
                    let lbl = get_label(labels, &list[1]);
                    let arg0 = Reg::from(list[2].expect_list());
                    let arg1 = Reg::from(list[3].expect_list());
                    Instruction::IsLt { lbl, arg0, arg1 }
                }
                "is_ge" => {
                    assert_eq!(list.len(), 4);
                    let lbl = get_label(labels, &list[1]);
                    let arg0 = Reg::from(list[2].expect_list());
                    let arg1 = Reg::from(list[3].expect_list());
                    Instruction::IsGe { lbl, arg0, arg1 }
                }
                "is_eq" => {
                    assert_eq!(list.len(), 4);
                    let lbl = get_label(labels, &list[1]);
                    let arg0 = Reg::from(list[2].expect_list());
                    let arg1 = Reg::from(list[3].expect_list());
                    Instruction::IsEq { lbl, arg0, arg1 }
                }
                "is_ne" => {
                    assert_eq!(list.len(), 4);
                    let lbl = get_label(labels, &list[1]);
                    let arg0 = Reg::from(list[2].expect_list());
                    let arg1 = Reg::from(list[3].expect_list());
                    Instruction::IsNe { lbl, arg0, arg1 }
                }
                "is_int" => {
                    assert_eq!(list.len(), 3);
                    let lbl = get_label(labels, &list[1]);
                    let arg = Reg::from(list[2].expect_list());
                    Instruction::IsInteger { lbl, arg }
                }
                "jmp" => {
                    assert_eq!(list.len(), 2);
                    let lbl = get_label(labels, &list[1]);
                    Instruction::Jmp { lbl }
                }
                "ret" => {
                    assert_eq!(list.len(), 1);
                    Instruction::Ret
                }
                "call" => {
                    assert_eq!(list.len(), 2);
                    let ip = get_label(labels, &list[1]);
                    Instruction::Call { ip }
                }
                "spawn" => {
                    assert_eq!(list.len(), 2);
                    let instrs = list[1]
                        .expect_list()
                        .iter()
                        .map(|l| Instruction::from((labels, l.expect_list())))
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
                    DataObject::Pid(PID::new(x[1].expect_num(), x[2].expect_num()))
                }
                _ => todo!(),
            },
        }
    }
}

pub fn parse_str(s: &str) -> Vec<Instruction> {
    let lexerdef = byte_l::lexerdef();
    let lexer = lexerdef.lexer(s);
    let (res, errs) = byte_y::parse(&lexer);
    for e in errs {
        println!("{}", e.pp(&lexer, &byte_y::token_epp));
    }

    let res = res.unwrap().ok().unwrap();

    // First pass to find labels
    let mut labels = Vec::new();
    let mut lines = Vec::new();
    for line in &res {
        if let Item::Atom(a) = &line[0] {
            if a == "label" {
                let n = line[1].expect_num();
                labels.push((n, lines.len()));
            } else {
                lines.push(line);
            }
        }
    }

    lines
        .into_iter()
        .map(|l| Instruction::from((&labels[..], l)))
        .collect()
}
