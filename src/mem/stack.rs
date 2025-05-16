use std::iter;

use super::DataObject;

#[derive(Debug)]
pub struct Stack {
    registers: Vec<DataObject>,
    call_frames: Vec<CallFrame>,
}

pub enum Reg {
    X(usize),
    Y(usize),
    Htop,
    E,
    I,
    FP,
    CP,
    #[allow(non_camel_case_types)]
    fcalls,
}

#[derive(Debug)]
pub struct CallFrame {
    /// pointer to function being called
    f: (),

    /// return instruction pointer
    ip: (),

    /// base pointer
    bp: (),
}

impl Stack {
    pub fn new() -> Self {
        Self {
            registers: Vec::new(),
            call_frames: Vec::new(),
        }
    }

    pub fn get(&self, reg: Reg) -> Result<&DataObject, &'static str> {
        match reg {
            Reg::X(_) => Err("cannot get X register from stack"),
            Reg::Y(i) => self.registers.get(i).ok_or("register out of bounds"),
            Reg::Htop => todo!(),
            Reg::E => todo!(),
            Reg::I => todo!(),
            Reg::FP => todo!(),
            Reg::CP => todo!(),
            Reg::fcalls => todo!(),
        }
    }

    pub fn put(&mut self, reg: Reg, data: DataObject) {
        match reg {
            Reg::X(_) => {}
            Reg::Y(i) => {
                if i >= self.registers.len() {
                    panic!("register Y{i} does not exist")
                }
                self.registers[i] = data;
            }
            Reg::Htop => todo!(),
            Reg::E => todo!(),
            Reg::I => todo!(),
            Reg::FP => todo!(),
            Reg::CP => todo!(),
            Reg::fcalls => todo!(),
        }
    }

    pub fn allocate(&mut self, words: usize) {
        self.registers
            .extend(iter::repeat_n(DataObject::Nil, words));
    }
}

#[cfg(test)]
mod tests {
    use crate::mem::{DataObject, stack::Reg};

    use super::Stack;

    fn registers() {
        let mut stack = Stack::new();
        stack.allocate(1);
        assert_eq!(stack.get(Reg::Y(0)), Ok(&DataObject::Nil));
        stack.put(Reg::Y(0), DataObject::Small(0));
        assert_eq!(stack.get(Reg::Y(0)), Ok(&DataObject::Small(0)));
    }
}
