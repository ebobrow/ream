use std::iter;

use crate::instr::Instruction;

use super::DataObject;

#[derive(Debug)]
pub struct Stack {
    registers: Vec<DataObject>,
    call_frames: Vec<CallFrame>,
    instrs: Vec<Instruction>,
}

#[derive(Debug, Clone)]
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
#[allow(dead_code)]
pub struct CallFrame {
    // pointer to function being called
    // only need if we have like external functions i think
    // f: (),
    //
    /// return instruction pointer
    ip: usize,

    /// base pointer
    bp: usize,
}

impl CallFrame {
    pub fn new(ip: usize, bp: usize) -> Self {
        Self { ip, bp }
    }
}

impl Stack {
    pub fn new_from(instrs: Vec<Instruction>) -> Self {
        Self {
            registers: vec![DataObject::Nil; 256],
            call_frames: vec![CallFrame::new(0, 0)],
            instrs,
        }
    }

    pub fn get(&self, reg: &Reg) -> Result<DataObject, String> {
        match reg {
            Reg::Y(i) => self
                .registers
                .get(*i)
                .ok_or("register out of bounds".to_string())
                .cloned(),
            Reg::CP => Ok(DataObject::IC(self.cur_frame().ip)),
            _ => Err(format!("cannot get {reg:?} from stack")),
        }
    }

    pub fn put(&mut self, reg: &Reg, data: DataObject) {
        match reg {
            Reg::Y(i) => {
                if *i >= self.registers.len() {
                    panic!("register Y{i} does not exist")
                }
                self.registers[*i] = data;
            }
            Reg::CP => {
                if let DataObject::IC(ip) = data {
                    self.cur_frame_mut().ip = ip;
                }
            }
            _ => panic!("cannot set {reg:?} from stack"),
        }
    }

    fn cur_frame(&self) -> &CallFrame {
        self.call_frames
            .last()
            .unwrap_or_else(|| panic!("stack cannot have empty call frame list"))
    }

    fn cur_frame_mut(&mut self) -> &mut CallFrame {
        self.call_frames
            .last_mut()
            .unwrap_or_else(|| panic!("stack cannot have empty call frame list"))
    }

    pub fn allocate(&mut self, words: usize) {
        self.registers
            .extend(iter::repeat_n(DataObject::Nil, words));
    }

    pub fn deallocate(&mut self, words: usize) {
        let regs = self.registers.len();
        if regs < words {
            panic!("cannot deallocate {words} registers");
        } else {
            self.registers = self.registers[..regs - words].to_vec();
        }
    }

    pub fn allocate_call(&mut self, ip: usize) {
        self.call_frames
            .push(CallFrame::new(ip, self.registers.len()));
        self.allocate(256);
    }

    pub fn ret(&mut self) -> bool {
        self.call_frames.pop();
        if self.call_frames.is_empty() {
            true
        } else {
            self.deallocate(256);
            false
        }
    }

    pub fn instrs(&self) -> &[Instruction] {
        &self.instrs
    }

    pub fn cp(&self) -> Option<usize> {
        self.call_frames.last().map(|frame| frame.ip)
    }
}

#[cfg(test)]
mod tests {
    use crate::mem::{DataObject, stack::Reg};

    use super::Stack;

    #[test]
    fn registers() {
        let mut stack = Stack::new_from(Vec::new());
        stack.allocate(1);
        assert_eq!(stack.get(&Reg::Y(0)), Ok(DataObject::Nil));
        stack.put(&Reg::Y(0), DataObject::Small(0));
        assert_eq!(stack.get(&Reg::Y(0)), Ok(DataObject::Small(0)));
    }
}
