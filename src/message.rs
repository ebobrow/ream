use std::sync::Arc;

use crate::DataObject;

#[derive(Debug)]
pub struct Mailbox {
    msgs: Vec<DataObject>,
    save: Option<usize>,
}

impl Mailbox {
    pub fn new() -> Self {
        Self {
            msgs: Vec::new(),
            save: None,
        }
    }

    pub fn add_msg(&mut self, msg: DataObject) {
        self.msgs.push(msg);
    }
}

pub type MBuf = Arc<DataObject>;

// pub struct Mailbox<'a> {
//     len: usize,
//     first: &'a Message<'a>,
//     last: &'a Message<'a>,
//     save: &'a Message<'a>,
// }

// pub struct Message<'a> {
//     next: &'a Message<'a>,
//     body: DataObject,
// }
