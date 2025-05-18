use std::{
    sync::{
        Arc, Mutex, MutexGuard,
        mpsc::{self, Sender},
    },
    thread,
};

use crate::{DataObject, Instruction, Reg, mem::stack::Stack, pcb::PCB, scheduler::Scheduler};

#[derive(Debug)]
pub struct VM {
    registers: Arc<Mutex<[DataObject; 1024]>>,
    schedulers: Vec<Sender<Arc<Mutex<Process>>>>,
}

impl VM {
    pub fn new() -> Self {
        let registers = Arc::new(Mutex::new(core::array::from_fn(|_| DataObject::Nil)));
        let mut schedulers = Vec::with_capacity(thread::available_parallelism().unwrap().get());
        for _ in 0..schedulers.capacity() {
            let (tx, rx) = mpsc::channel::<Arc<Mutex<Process>>>();
            schedulers.push(tx);
            thread::spawn(move || {
                Scheduler::new(rx).run();
            });
        }

        Self {
            registers: registers.clone(),
            schedulers,
        }
    }

    // TODO: this seems... wrong
    pub fn run_instrs(&mut self, instrs: Vec<Instruction>) {
        // TODO: proper id
        let process = Process::new(0, instrs, self.registers.clone());
        self.schedulers[0]
            .send(Arc::new(Mutex::new(process)))
            .unwrap();
    }

    // fn spawn(&mut self, instrs: Vec<Instruction>) {
    //     self.processes.push(Arc::new(Mutex::new(Process::new(
    //         self.processes.len().try_into().unwrap(),
    //         instrs,
    //         self.registers.clone(),
    //     ))));
    // }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Process {
    stack: Stack,
    registers: Arc<Mutex<[DataObject; 1024]>>,
    // heap: Heap,
    // message_area: (),
    pcb: PCB,
}

impl Process {
    fn new(id: u32, instrs: Vec<Instruction>, registers: Arc<Mutex<[DataObject; 1024]>>) -> Self {
        Self {
            stack: Stack::new_from(instrs),
            registers,
            pcb: PCB::new(id),
            // heap: Vec::new(),
            // message_area: (),
        }
    }

    pub fn id(&self) -> &DataObject {
        self.pcb.id()
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
            Reg::I => f(Some(&DataObject::IC(self.pcb.get_ip()))),
            Reg::fcalls => f(Some(&DataObject::Small(
                self.pcb.get_fcalls().try_into().unwrap(),
            ))),
            Reg::Y(_) => f(self.stack.get(reg).ok()),

            Reg::Htop | Reg::E | Reg::FP | Reg::CP => todo!(),
        }
    }

    fn put(&mut self, reg: &Reg, data: DataObject) {
        match reg {
            Reg::X(i) => {
                if *i < 1024 {
                    let mut registers = self.registers.lock().unwrap();
                    registers[*i] = data;
                }
            }
            Reg::I | Reg::fcalls => panic!("we probably don't want to allow this"),
            Reg::Y(_) => self.stack.put(reg, data),

            Reg::Htop | Reg::E | Reg::FP | Reg::CP => todo!(),
        }
    }

    fn comparison(&mut self, arg0: &Reg, arg1: &Reg, offset: usize, op: impl Fn(u32, u32) -> bool) {
        let a = self.get(arg0, |i| i.unwrap().expect_int());
        let b = self.get(arg1, |i| i.unwrap().expect_int());
        if op(a, b) {
            self.pcb.inc_ip(offset);
        }
    }

    fn type_test(&mut self, arg: &Reg, offset: usize, test: impl Fn(&DataObject) -> bool) {
        if self.get(arg, |a| test(a.unwrap())) {
            self.pcb.inc_ip(offset);
        }
    }

    pub fn run(&mut self) {
        self.pcb.set_running();
        while self.pcb.get_ip() < self.stack.instrs().len() {
            match self.stack.instrs()[self.pcb.get_ip()].clone() {
                Instruction::Move { dest, src } => {
                    self.put(&dest, src);
                }
                Instruction::Add { arg0, arg1, ret } => {
                    self.put(
                        &ret,
                        DataObject::Small(
                            self.get(&arg0, |i| i.unwrap().expect_int())
                                + self.get(&arg1, |i| i.unwrap().expect_int()),
                        ),
                    );
                }
                Instruction::Allocate { stack_need } => self.stack.allocate(stack_need),

                // TODO: make these work for other types
                Instruction::IsLt { lbl, arg0, arg1 } => {
                    self.comparison(&arg0, &arg1, lbl, |a, b| a < b)
                }
                Instruction::IsGe { lbl, arg0, arg1 } => {
                    self.comparison(&arg0, &arg1, lbl, |a, b| a >= b)
                }
                Instruction::IsEq { lbl, arg0, arg1 } => {
                    self.comparison(&arg0, &arg1, lbl, |a, b| a == b)
                }
                Instruction::IsNe { lbl, arg0, arg1 } => {
                    self.comparison(&arg0, &arg1, lbl, |a, b| a != b)
                }

                Instruction::IsInteger { lbl, arg } => {
                    self.type_test(&arg, lbl, |a| matches!(a, DataObject::Small(_)))
                }

                Instruction::Ret => {
                    if self.stack.ret() {
                        break;
                    }
                }
                Instruction::Call { ip } => {
                    self.stack.allocate_call(ip);
                    if self.pcb.dec_fcalls() {
                        break;
                    }
                }
            }
            self.pcb.inc_ip(1);
        }
    }

    pub fn pcb(&self) -> &PCB {
        &self.pcb
    }

    pub fn pcb_mut(&mut self) -> &mut PCB {
        &mut self.pcb
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        instr::Instruction,
        mem::{DataObject, stack::Reg},
        vm::Process,
    };

    fn run_test<const I: usize, const R: usize>(
        instrs: [Instruction; I],
        regs: [(Reg, DataObject); R],
    ) {
        let registers = Arc::new(Mutex::new(core::array::from_fn(|_| DataObject::Nil)));
        let mut process = Process::new(0, instrs.to_vec(), registers);
        process.run();
        for (reg, value) in regs {
            process.get(&reg, |v| {
                assert_eq!(v, Some(&value));
            })
        }
    }

    #[test]
    fn basic() {
        run_test(
            [
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
            [
                (Reg::X(0), DataObject::Small(12)),
                (Reg::X(1), DataObject::Small(2)),
            ],
        );
    }

    #[test]
    fn memory() {
        run_test(
            [
                Instruction::Allocate { stack_need: 2 },
                Instruction::Move {
                    dest: Reg::Y(0),
                    src: DataObject::Small(0),
                },
            ],
            [
                (Reg::Y(0), DataObject::Small(0)),
                (Reg::Y(1), DataObject::Nil),
            ],
        );
    }

    #[test]
    fn comparisons() {
        run_test(
            [
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
            [
                (Reg::X(0), DataObject::Small(1)),
                (Reg::X(1), DataObject::Small(42)),
            ],
        );

        run_test(
            [
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
            [
                (Reg::X(0), DataObject::Small(42)),
                (Reg::X(1), DataObject::Small(42)),
            ],
        );
    }

    #[test]
    fn type_test() {
        run_test(
            [
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
            [(Reg::X(0), DataObject::Small(0))],
        );

        run_test(
            [
                Instruction::IsInteger {
                    lbl: 1,
                    arg: Reg::X(0),
                },
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(0),
                },
            ],
            [(Reg::X(0), DataObject::Small(0))],
        );
    }

    #[test]
    fn calls() {
        run_test(
            [
                Instruction::Call { ip: 2 },
                Instruction::Ret,
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(0),
                },
                Instruction::Ret,
            ],
            [(Reg::X(0), DataObject::Small(0))],
        );

        run_test(
            [
                Instruction::Ret,
                Instruction::Move {
                    dest: Reg::X(0),
                    src: DataObject::Small(0),
                },
                Instruction::Ret,
            ],
            [(Reg::X(0), DataObject::Nil)],
        );
    }
}
