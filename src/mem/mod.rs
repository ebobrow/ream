pub mod heap;
pub mod stack;

#[derive(Debug, Clone, PartialEq)]
pub enum DataObject {
    Small(u32),
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

impl DataObject {
    pub fn expect_int(&self) -> u32 {
        if let DataObject::Small(v) = self {
            *v
        } else {
            panic!("expected int, got {:?}", self);
        }
    }
}
