use super::DataObject;

#[derive(Debug)]
pub struct Stack(Vec<DataObject>);

pub enum Reg {
    X(usize),
    Y(usize),
    Htop,
    E,
    I,
    FP,
    CP,
    fcalls,
}

impl Stack {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get(&self, reg: Reg) -> Option<&DataObject> {
        todo!()
    }

    pub fn put(&mut self, reg: Reg, data: DataObject) {
        todo!()
    }
}
