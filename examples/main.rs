use ream::{DataObject, Instruction, Reg, VM};

fn main() {
    let mut vm = VM::new();
    vm.run_instrs(vec![Instruction::Move {
        dest: Reg::X(0),
        src: DataObject::Small(0),
    }]);

    // println!("{vm:#?}")
}
