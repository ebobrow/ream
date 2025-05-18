use std::{
    sync::{Arc, Mutex, mpsc::Receiver},
    thread,
};

use crate::vm::Process;

pub enum Message {
    Kill,
    Spawn(Arc<Mutex<Process>>),
}

#[derive(Debug)]
pub struct Scheduler {
    // TODO: three queues based on priority
    ready_queue_first: Option<Arc<Mutex<Process>>>,
    ready_queue_last: Option<Arc<Mutex<Process>>>,
    // num_processes: AtomicUsize,
    rx: Receiver<Message>,
}

impl Scheduler {
    pub fn new(rx: Receiver<Message>) -> Self {
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
            if let Ok(msg) = self.rx.try_recv() {
                match msg {
                    Message::Kill => break,
                    Message::Spawn(process) => {
                        println!("received process");
                        self.push_ready_queue(process);
                    }
                }
            } else if let Some(first) = self.pop_ready_queue() {
                let done = {
                    let first = first.clone();
                    let mut first = first.lock().unwrap();
                    if first.pcb().is_runnable() {
                        println!("running process {:?}", first.id());
                        first.run()
                    } else {
                        false
                    }
                };
                if !done {
                    self.push_ready_queue(first);
                } else {
                    println!("process finished!");
                }
            } else {
                thread::yield_now();
            }
        }
    }
}
