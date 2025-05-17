use std::sync::{Arc, Mutex};

use crate::{Process, mem::DataObject};

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct PCB {
    id: DataObject,
    ip: usize,
    fcalls: usize,
    status: State,

    /// Suspend count
    rstatus: usize,

    next: Option<Arc<Mutex<Process>>>,
}

impl PCB {
    pub fn new(id: u32) -> Self {
        Self {
            id: DataObject::Pid(id),
            ip: 0,
            fcalls: 4000,
            status: State::Runnable,
            rstatus: 0,
            next: None,
        }
    }

    pub fn get_ip(&self) -> usize {
        self.ip
    }

    pub fn inc_ip(&mut self, offset: usize) {
        self.ip += offset;
    }

    pub fn get_fcalls(&self) -> usize {
        self.fcalls
    }

    pub fn dec_fcalls(&mut self) -> Option<Arc<Mutex<Process>>> {
        assert_eq!(self.status, State::Running);
        self.fcalls -= 1;
        if self.fcalls == 0 {
            self.status = State::Runnable;
            return self.next.clone();
        }
        None
    }

    pub fn is_runnable(&self) -> bool {
        matches!(self.status, State::Runnable)
    }

    pub fn suspend(&mut self) {
        self.rstatus += 1;
        match self.status {
            State::Running | State::Runnable => self.status = State::Suspended { runnable: true },
            State::Waiting => self.status = State::Suspended { runnable: false },
            State::Suspended { .. } => {}

            State::Free | State::Exiting | State::Garbing { .. } => todo!(),
        }
    }

    pub fn resume(&mut self) {
        if let State::Suspended { runnable } = self.status {
            self.rstatus -= 1;
            if self.rstatus == 0 {
                if runnable {
                    self.status = State::Runnable;
                } else {
                    self.status = State::Waiting;
                }
            }
        } else {
            panic!("resume called on non-suspended process");
        }
    }

    pub fn set_next(&mut self, next: Arc<Mutex<Process>>) {
        assert!(self.next.is_none());
        self.next = Some(next);
    }

    pub fn set_running(&mut self) {
        assert_eq!(self.status, State::Runnable);
        self.status = State::Running;
    }

    pub fn next(&self) -> Option<&Arc<Mutex<Process>>> {
        self.next.as_ref()
    }
}

#[derive(Debug, PartialEq)]
enum State {
    // If a suspended waiting process receives a timeout rstatus is set to runnable so it will
    // resume as runnable
    Suspended { runnable: bool },
    Runnable,
    Free,
    Exiting,
    Running,
    Waiting,
    Garbing { old_status: Box<State> },
}
