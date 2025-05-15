// TODO: this is bad; for now the heap is just a vec indexed by usizes
pub type Heap = Vec<DataObject>;
pub type Pointer = usize;

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

#[derive(Debug, Clone)]
pub struct DataObject {
    tag: Tag,
    // actually it's the remaining 18 bits after the 4 bit tag
    value: u32,
}

impl DataObject {
    pub fn new_nil() -> Self {
        Self {
            tag: Tag::Nil,
            value: 0,
        }
    }

    pub fn new_int(value: u32) -> Self {
        Self {
            tag: Tag::Small,
            value,
        }
    }

    pub fn expect_int(&self) -> u32 {
        if let Tag::Small = self.tag {
            self.value
        } else {
            panic!("expected int, got {:?}", self.tag);
        }
    }
}

#[derive(Debug, Clone)]
enum Tag {
    Small,
    Big,
    Float,
    Atom,
    Refer,
    Port,
    Pid,
    Tuple,
    Nil,
    List,
    Arityval,
    Moved,
    Catch,
    Thing,
    Binary,
    Blank,
    IC,

    CP0,
    CP4,
    CP8,
    CP12,
}
