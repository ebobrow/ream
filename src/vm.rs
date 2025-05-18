use std::{
    sync::{
        Arc, Mutex, MutexGuard,
        mpsc::{self, Sender},
    },
    thread,
};

use crate::{
    DataObject, Instruction, Reg,
    mem::{PID, Registers, stack::Stack},
    pcb::PCB,
    scheduler::{Message, Scheduler},
};

#[derive(Debug)]
pub struct VM {
    registers: Registers,
    schedulers: Vec<Sender<Message>>,
}

impl VM {
    pub fn new() -> Self {
        let registers = Arc::new(Mutex::new(core::array::from_fn(|_| DataObject::Nil)));
        let mut schedulers = Vec::with_capacity(thread::available_parallelism().unwrap().get());
        for i in 0..schedulers.capacity() {
            let (tx, rx) = mpsc::channel::<Message>();
            schedulers.push(tx);
            thread::spawn(move || {
                Scheduler::new(i, rx).run();
            });
        }

        Self {
            registers: registers.clone(),
            schedulers,
        }
    }

    pub fn spawn(&mut self, instrs: Vec<Instruction>) {
        let tx = &self.schedulers[0];
        tx.send(Message::Spawn((instrs, self.registers.clone(), tx.clone())))
            .unwrap();
    }

    pub fn wait(&self) {
        for tx in &self.schedulers {
            // Current behavior is for thread to stop after finishing remaining tasks
            tx.send(Message::Kill).unwrap();
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Process {
    stack: Stack,
    registers: Registers,
    // heap: Heap,
    // message_area: (),
    pcb: PCB,
    // TODO: this is weird and also doesn't account for the fact that it may be moved to a
    // different thread
    tx: Sender<Message>,
}

impl Process {
    pub fn new(
        id: PID,
        instrs: Vec<Instruction>,
        registers: Registers,
        tx: Sender<Message>,
    ) -> Self {
        Self {
            stack: Stack::new_from(instrs),
            registers,
            pcb: PCB::new(id),
            // heap: Vec::new(),
            // message_area: (),
            tx,
        }
    }

    pub fn id(&self) -> &DataObject {
        self.pcb.id()
    }

    // pub fn debug_regs(&self) {
    //     println!("{:#?}", &self.registers.lock().unwrap()[0..5]);
    // }

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
            self.pcb.set_ip(offset);
        }
    }

    fn type_test(&mut self, arg: &Reg, offset: usize, test: impl Fn(&DataObject) -> bool) {
        if self.get(arg, |a| test(a.unwrap())) {
            self.pcb.set_ip(offset);
        }
    }

    /// returns true if process has finished
    pub fn run(&mut self) -> bool {
        self.pcb.set_running();
        while self.pcb.get_ip() < self.stack.instrs().len() {
            let instr = self.stack.instrs()[self.pcb.get_ip()].clone();
            self.pcb.inc_ip(1);
            // println!("{instr:?}");
            match instr {
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
                    self.pcb.set_ip(self.stack.cp().unwrap());
                    if self.stack.ret() {
                        return true;
                    }
                }
                Instruction::Call { ip } => {
                    self.stack.allocate_call(ip);
                    self.pcb.set_ip(self.pcb.get_ip());
                    if self.pcb.dec_fcalls() {
                        return false;
                    }
                }
                Instruction::Jmp { lbl } => self.pcb.set_ip(lbl),
                Instruction::Spawn { instrs } => {
                    self.tx
                        .send(Message::Spawn((
                            instrs,
                            self.registers.clone(),
                            self.tx.clone(),
                        )))
                        .unwrap();
                }
            }
        }
        true
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
    use std::sync::{Arc, Mutex, mpsc};

    use crate::{
        instr::Instruction,
        mem::{DataObject, PID, stack::Reg},
        vm::Process,
    };

    fn run_test<const I: usize, const R: usize>(
        instrs: [Instruction; I],
        regs: [(Reg, DataObject); R],
    ) {
        let (tx, _) = mpsc::channel();
        let registers = Arc::new(Mutex::new(core::array::from_fn(|_| DataObject::Nil)));
        let mut process = Process::new(PID::new(0, 0), instrs.to_vec(), registers, tx);
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
                    lbl: 4,
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
                    lbl: 4,
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
                    lbl: 3,
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
                    lbl: 3,
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
