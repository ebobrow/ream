use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

pub mod heap;
pub mod stack;

// TODO: tagged pointers https://rust-hosted-langs.github.io/book/chapter-interp-tagged-ptrs.html
#[derive(Debug, Clone, PartialEq)]
pub enum DataObject {
    Small(u32),
    Big,
    Float,
    Atom,
    Refer,
    Port,
    Pid(PID),
    Tuple,
    Nil,
    List,
    Arityval,
    Moved,
    Catch,
    Thing,
    Binary,
    Blank,
    IC(usize),

    CP0,
    CP4,
    CP8,
    CP12,
}

impl DataObject {
    pub fn expect_int(&self) -> u32 {
        if let DataObject::Small(v) = self {
            *v
        } else {
            panic!("expected int, got {self:?}");
        }
    }

    pub fn expect_pid(&self) -> &PID {
        if let DataObject::Pid(pid) = self {
            pid
        } else {
            panic!("expected int, got {self:?}");
        }
    }
}

// TODO: this is probably bad
#[derive(Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub struct PID {
    scheduler: usize,
    num: usize,
}

impl PID {
    pub fn new(scheduler: usize, num: usize) -> Self {
        Self { scheduler, num }
    }
}

impl Debug for PID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>{}", self.scheduler, self.num)
    }
}

pub type Registers = Arc<Mutex<[DataObject; 1024]>>;
