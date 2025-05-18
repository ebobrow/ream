use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
};

use crate::{
    Instruction,
    mem::{PID, Registers},
    vm::Process,
};

pub enum Message {
    Kill,
    Spawn((Vec<Instruction>, Registers, mpsc::Sender<Message>)),
}

#[derive(Debug)]
pub struct Scheduler {
    // TODO: three queues based on priority
    ready_queue_first: Option<Arc<Mutex<Process>>>,
    ready_queue_last: Option<Arc<Mutex<Process>>>,
    procs_recvd: usize,
    rx: Receiver<Message>,
    id: usize,
}

impl Scheduler {
    pub fn new(id: usize, rx: Receiver<Message>) -> Self {
        Self {
            ready_queue_first: None,
            ready_queue_last: None,
            procs_recvd: 0,
            rx,
            id,
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
        if let Some(old_last) = &self.ready_queue_last {
            old_last.lock().unwrap().pcb_mut().set_next(process.clone());
        } else {
            self.ready_queue_first = Some(process.clone());
        }
        self.ready_queue_last = Some(process);
    }

    pub fn run(&mut self) {
        let mut kill_recvd = false;
        loop {
            if let Ok(msg) = self.rx.try_recv() {
                match msg {
                    Message::Kill => kill_recvd = true,
                    Message::Spawn((instrs, regs, tx)) => {
                        let pid = PID::new(self.id, self.procs_recvd);
                        println!("received process id {pid:?}");
                        self.procs_recvd += 1;
                        self.push_ready_queue(Arc::new(Mutex::new(Process::new(
                            pid, instrs, regs, tx,
                        ))));
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
            } else if kill_recvd {
                break;
            } else {
                thread::yield_now();
            }
        }
    }
}
