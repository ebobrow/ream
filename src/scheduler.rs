use std::{
    sync::{Arc, Mutex, mpsc::Receiver},
    thread,
};

use crate::vm::Process;

#[derive(Debug)]
pub struct Scheduler {
    // TODO: three queues based on priority
    ready_queue_first: Option<Arc<Mutex<Process>>>,
    ready_queue_last: Option<Arc<Mutex<Process>>>,
    // num_processes: AtomicUsize,
    rx: Receiver<Arc<Mutex<Process>>>,
}

impl Scheduler {
    pub fn new(rx: Receiver<Arc<Mutex<Process>>>) -> Self {
        Self {
            ready_queue_first: None,
            ready_queue_last: None,
            // num_processes: AtomicUsize::new(0),
            rx,
        }
    }

    fn pop_ready_queue(&mut self) -> Option<Arc<Mutex<Process>>> {
        let old_first = self.ready_queue_first.clone();
        if let Some(ref old_first) = old_first {
            let next = old_first.lock().unwrap().pcb().next().cloned();
            if next.is_none() {
                self.ready_queue_last = None;
            }
            self.ready_queue_first = next;
        }
        old_first
    }

    fn push_ready_queue(&mut self, process: Arc<Mutex<Process>>) {
        // *self.num_processes.get_mut() += 1;

        if let Some(old_last) = &self.ready_queue_last {
            old_last.lock().unwrap().pcb_mut().set_next(process.clone());
        } else {
            self.ready_queue_first = Some(process.clone());
        }
        self.ready_queue_last = Some(process);
    }

    pub fn run(&mut self) {
        loop {
            if let Some(first) = self.pop_ready_queue() {
                let first = first.clone();
                let mut first = first.lock().unwrap();
                first.run();
            } else if let Ok(process) = self.rx.try_recv() {
                self.push_ready_queue(process);
            } else {
                thread::yield_now();
            }
        }
    }
}
