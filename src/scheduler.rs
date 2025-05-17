use std::sync::{Arc, Mutex};

use crate::Process;

#[derive(Debug)]
pub struct Scheduler {
    // TODO: three queues based on priority
    ready_queue_first: Arc<Mutex<Process>>,
    ready_queue_last: Arc<Mutex<Process>>,
}

impl Scheduler {
    pub fn new(processs: Arc<Mutex<Process>>) -> Self {
        Self {
            ready_queue_first: processs.clone(),
            ready_queue_last: processs,
        }
    }

    pub fn run(&mut self) {
        self.ready_queue_first.lock().unwrap().resume();
    }

    pub fn enqueue(&mut self, processs: Arc<Mutex<Process>>) {
        self.ready_queue_last
            .lock()
            .unwrap()
            .pcb
            .set_next(processs.clone());
        self.ready_queue_last = processs;
    }
}
