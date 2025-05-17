use std::{
    sync::{Arc, Mutex, mpsc::Receiver},
    thread,
};

use crate::Process;

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
            self.ready_queue_first = old_first.lock().unwrap().pcb.next().cloned();
        }
        old_first
    }

    pub fn run(&mut self) {
        loop {
            if let Some(first) = self.pop_ready_queue() {
                let first = first.clone();
                self.run_process(&mut first.lock().unwrap());
            } else if let Ok(process) = self.rx.try_recv() {
                self.enqueue(process);
            } else {
                thread::yield_now();
            }
        }
    }

    fn run_process(&mut self, process: &mut Process) {
        let res = process.run();

        if let Ok(process) = self.rx.try_recv() {
            self.enqueue(process);
        }

        match res {
            Some(next) => {
                let mut next = next.lock().unwrap();
                if next.pcb.is_runnable() {
                    self.run_process(&mut next);
                }
            }
            None => {
                // self.re
            }
        }
    }

    pub fn enqueue(&mut self, process: Arc<Mutex<Process>>) {
        // *self.num_processes.get_mut() += 1;

        if let Some(old_last) = &self.ready_queue_last {
            old_last.lock().unwrap().pcb.set_next(process.clone());
        } else {
            self.ready_queue_first = Some(process.clone());
        }
        self.ready_queue_last = Some(process);
    }
}
