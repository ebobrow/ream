use std::sync::{Arc, Mutex};

use heap::{DataObject, Reg, Stack};
use instr::Instruction;

mod heap;
mod instr;

#[derive(Debug)]
struct VM {
    registers: Arc<Mutex<[DataObject; 1024]>>,
    processes: Vec<Process>,
}

impl VM {
    fn new() -> Self {
        let registers = Arc::new(Mutex::new(core::array::from_fn(|_| DataObject::new_nil())));
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
    // heap: Heap,
    // message_area: (),
    // pcb: (),
}

impl Process {
    fn new(registers: Arc<Mutex<[DataObject; 1024]>>) -> Self {
        Self {
            stack: Stack::new(),
            registers,
            // heap: Vec::new(),
            // message_area: (),
            // pcb: (),
        }
    }

    fn get(&self, reg: Reg) -> Option<DataObject> {
        match reg {
            Reg::X(i) => {
                if i < 1024 {
                    let registers = self.registers.lock().unwrap();
                    Some(registers[i].clone())
                } else {
                    None
                }
            }
            Reg::Y(_) => todo!(),
            Reg::Htop => todo!(),
            Reg::E => todo!(),
            Reg::I => todo!(),
            Reg::FP => todo!(),
            Reg::CP => todo!(),
            Reg::fcalls => todo!(),
        }
    }

    fn put(&mut self, reg: Reg, data: DataObject) {
        match reg {
            Reg::X(i) => {
                if i < 1024 {
                    let mut registers = self.registers.lock().unwrap();
                    registers[i] = data;
                }
            }
            Reg::Y(_) => todo!(),
            Reg::Htop => todo!(),
            Reg::E => todo!(),
            Reg::I => todo!(),
            Reg::FP => todo!(),
            Reg::CP => todo!(),
            Reg::fcalls => todo!(),
        }
    }

    fn run(&mut self, instrs: Vec<Instruction>) {
        for instr in instrs {
            match instr {
                Instruction::Move { dest, src } => {
                    self.put(dest, DataObject::new_int(src));
                }
                Instruction::Add { arg0, arg1, ret } => {
                    self.put(
                        ret,
                        DataObject::new_int(
                            self.get(arg0).unwrap().expect_int()
                                + self.get(arg1).unwrap().expect_int(),
                        ),
                    );
                }
            }
        }
    }
}

fn main() {
    let mut vm = VM::new();
    vm.run(vec![
        Instruction::Move {
            dest: Reg::X(0),
            src: 10,
        },
        Instruction::Move {
            dest: Reg::X(1),
            src: 2,
        },
        Instruction::Add {
            arg0: Reg::X(0),
            arg1: Reg::X(1),
            ret: Reg::X(0),
        },
    ]);
    println!("{:#?}", vm);
}
