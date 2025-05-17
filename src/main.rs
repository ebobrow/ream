#![allow(dead_code)]
#![feature(mapped_lock_guards)]

use std::sync::{Arc, Mutex, MutexGuard};

use instr::Instruction;
use mem::{
    DataObject,
    stack::{Reg, Stack},
};

mod instr;
mod mem;

#[derive(Debug)]
struct VM {
    registers: Arc<Mutex<[DataObject; 1024]>>,
    processes: Vec<Process>,
}

impl VM {
    fn new() -> Self {
        let registers = Arc::new(Mutex::new(core::array::from_fn(|_| DataObject::Nil)));
        Self {
            registers: registers.clone(),
            processes: vec![Process::new(registers.clone())],
        }
    }

    fn run(&mut self, instrs: Vec<Instruction>) {
        self.processes[0].run(instrs);
    }

    fn spawn(&mut self) {
        self.processes.push(Process::new(self.registers.clone()));
    }
}

#[derive(Debug)]
struct Process {
    stack: Stack,
    registers: Arc<Mutex<[DataObject; 1024]>>,
    ip: usize,
    // heap: Heap,
    // message_area: (),
    // pcb: (),
}

impl Process {
    fn new(registers: Arc<Mutex<[DataObject; 1024]>>) -> Self {
        Self {
            stack: Stack::new(),
            registers,
            ip: 0,
            // heap: Vec::new(),
            // message_area: (),
            // pcb: (),
        }
    }

    fn get<T, U: FnOnce(Option<&DataObject>) -> T>(&self, reg: &Reg, f: U) -> T {
        match reg {
            Reg::X(i) => {
                if *i < 1024 {
                    f(Some(&*MutexGuard::map(
                        self.registers.lock().unwrap(),
                        |arr| &mut arr[*i],
                    )))
                } else {
                    f(None)
                }
            }
            Reg::I => f(Some(&DataObject::IC(self.ip))),
            _ => f(self.stack.get(reg).ok()),
        }
    }

    fn put(&mut self, reg: &Reg, data: DataObject) {
        if let Reg::X(i) = reg {
            if *i < 1024 {
                let mut registers = self.registers.lock().unwrap();
                registers[*i] = data;
            }
        } else {
            self.stack.put(reg, data);
        }
    }

    fn comparison(&mut self, arg0: &Reg, arg1: &Reg, offset: usize, op: impl Fn(u32, u32) -> bool) {
        let a = self.get(arg0, |i| i.unwrap().expect_int());
        let b = self.get(arg1, |i| i.unwrap().expect_int());
        if op(a, b) {
            self.ip += offset;
        }
    }

    fn type_test(&mut self, arg: &Reg, offset: usize, test: impl Fn(&DataObject) -> bool) {
        if self.get(arg, |a| test(a.unwrap())) {
            self.ip += offset;
        }
    }

    fn run(&mut self, instrs: Vec<Instruction>) {
        while self.ip < instrs.len() {
            match &instrs[self.ip] {
                Instruction::Move { dest, src } => {
                    // TODO: wish we didn't have to clone
                    self.put(dest, src.clone());
                }
                Instruction::Add { arg0, arg1, ret } => {
                    self.put(
                        ret,
                        DataObject::Small(
                            self.get(arg0, |i| i.unwrap().expect_int())
                                + self.get(arg1, |i| i.unwrap().expect_int()),
                        ),
                    );
                }
                Instruction::Allocate { stack_need } => self.stack.allocate(*stack_need),

                // TODO: make these work for other types
                Instruction::IsLt { lbl, arg0, arg1 } => {
                    self.comparison(arg0, arg1, *lbl, |a, b| a < b)
                }
                Instruction::IsGe { lbl, arg0, arg1 } => {
                    self.comparison(arg0, arg1, *lbl, |a, b| a >= b)
                }
                Instruction::IsEq { lbl, arg0, arg1 } => {
                    self.comparison(arg0, arg1, *lbl, |a, b| a == b)
                }
                Instruction::IsNe { lbl, arg0, arg1 } => {
                    self.comparison(arg0, arg1, *lbl, |a, b| a != b)
                }

                Instruction::IsInteger { lbl, arg } => {
                    self.type_test(arg, *lbl, |a| matches!(a, DataObject::Small(_)))
                }

                Instruction::Ret => {
                    if self.stack.ret() {
                        break;
                    }
                }
                Instruction::Call { ip } => self.stack.call(*ip),
            }
            self.ip += 1;
        }
    }
}

fn main() {
    let mut vm = VM::new();
    vm.run(vec![
        Instruction::Call { ip: 2 },
        Instruction::Ret,
        Instruction::Move {
            dest: Reg::X(0),
            src: DataObject::Small(0),
        },
        Instruction::Ret,
    ]);
    println!("{vm:#?}")
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        Process,
        instr::Instruction,
        mem::{DataObject, stack::Reg},
    };

    fn run_test(instrs: Vec<Instruction>, regs: Vec<(Reg, DataObject)>) {
        let registers = Arc::new(Mutex::new(core::array::from_fn(|_| DataObject::Nil)));
        let mut process = Process::new(registers);
        process.run(instrs);
        for (reg, value) in regs {
            process.get(&reg, |v| {
                assert_eq!(v, Some(&value));
            })
        }
    }

    #[test]
    fn basic() {
        run_test(
            vec![
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(10),
                },
                Instruction::Move {
                    dest: Reg::X(1),
                    src: DataObject::Small(2),
                },
                Instruction::Add {
                    arg0: Reg::X(0),
                    arg1: Reg::X(1),
                    ret: Reg::X(0),
                },
            ],
            vec![
                (Reg::X(0), DataObject::Small(12)),
                (Reg::X(1), DataObject::Small(2)),
            ],
        );
    }

    #[test]
    fn memory() {
        run_test(
            vec![
                Instruction::Allocate { stack_need: 2 },
                Instruction::Move {
                    dest: Reg::Y(0),
                    src: DataObject::Small(0),
                },
            ],
            vec![
                (Reg::Y(0), DataObject::Small(0)),
                (Reg::Y(1), DataObject::Nil),
            ],
        );
    }

    #[test]
    fn comparisons() {
        run_test(
            vec![
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(1),
                },
                Instruction::Move {
                    dest: Reg::X(1),
                    src: DataObject::Small(2),
                },
                Instruction::IsLt {
                    lbl: 1,
                    arg0: Reg::X(0),
                    arg1: Reg::X(1),
                },
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(42),
                },
                Instruction::Move {
                    dest: Reg::X(1),
                    src: DataObject::Small(42),
                },
            ],
            vec![
                (Reg::X(0), DataObject::Small(1)),
                (Reg::X(1), DataObject::Small(42)),
            ],
        );

        run_test(
            vec![
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(2),
                },
                Instruction::Move {
                    dest: Reg::X(1),
                    src: DataObject::Small(2),
                },
                Instruction::IsLt {
                    lbl: 1,
                    arg0: Reg::X(0),
                    arg1: Reg::X(1),
                },
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(42),
                },
                Instruction::Move {
                    dest: Reg::X(1),
                    src: DataObject::Small(42),
                },
            ],
            vec![
                (Reg::X(0), DataObject::Small(42)),
                (Reg::X(1), DataObject::Small(42)),
            ],
        );
    }

    #[test]
    fn type_test() {
        run_test(
            vec![
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(0),
                },
                Instruction::IsInteger {
                    lbl: 1,
                    arg: Reg::X(0),
                },
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Nil,
                },
            ],
            vec![(Reg::X(0), DataObject::Small(0))],
        );

        run_test(
            vec![
                Instruction::IsInteger {
                    lbl: 1,
                    arg: Reg::X(0),
                },
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(0),
                },
            ],
            vec![(Reg::X(0), DataObject::Small(0))],
        );
    }

    #[test]
    fn calls() {
        run_test(
            vec![
                Instruction::Call { ip: 2 },
                Instruction::Ret,
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(0),
                },
                Instruction::Ret,
            ],
            vec![(Reg::X(0), DataObject::Small(0))],
        );

        run_test(
            vec![
                Instruction::Ret,
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(0),
                },
                Instruction::Ret,
            ],
            vec![(Reg::X(0), DataObject::Nil)],
        );
    }
}
