pub mod heap;
pub mod stack;

// TODO: tagged pointers https://rust-hosted-langs.github.io/book/chapter-interp-tagged-ptrs.html
#[derive(Debug, Clone, PartialEq)]
pub enum DataObject {
    Small(u32),
    Big,
    Float,
    Atom,
    Refer,
    Port,
    Pid(u32),
    Tuple,
    Nil,
    List,
    Arityval,
    Moved,
    Catch,
    Thing,
    Binary,
    Blank,
    IC(usize),

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
            panic!("expected int, got {self:?}");
        }
    }
}
