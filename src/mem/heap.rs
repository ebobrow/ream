use super::DataObject;

// TODO: this is bad; for now the heap is just a vec indexed by usizes
pub type Heap = Vec<DataObject>;
pub type Pointer = usize;
