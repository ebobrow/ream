use std::{thread, time::Duration};

use ream::{DataObject, Instruction, PID, Reg, VM};

fn main() {
    let vm = VM::new();
    let proc1 = vec![
        Instruction::Move {
            dest: Reg::X(0),
            src: DataObject::Small(0),
        },
        Instruction::Move {
            dest: Reg::X(1),
            src: DataObject::Small(8001),
        },
        Instruction::Move {
            dest: Reg::X(2),
            src: DataObject::Small(1),
        },
        Instruction::Call { ip: 5 },
        Instruction::Ret,
        // Function that increments X0
        Instruction::Add {
            arg0: Reg::X(0),
            arg1: Reg::X(2),
            ret: Reg::X(0),
        },
        Instruction::IsEq {
            lbl: 8,
            arg0: Reg::X(0),
            arg1: Reg::X(1),
        },
        Instruction::Call { ip: 5 },
        Instruction::Ret,
    ];
    let proc2 = vec![
        Instruction::Spawn { instrs: proc1 },
        Instruction::Send {
            pid: DataObject::Pid(PID::new(0, 1)),
            data: DataObject::Nil,
        },
        Instruction::Move {
            dest: Reg::Y(0),
            src: DataObject::Small(0),
        },
        Instruction::Move {
            dest: Reg::Y(1),
            src: DataObject::Small(5000),
        },
        Instruction::Move {
            dest: Reg::Y(2),
            src: DataObject::Small(1),
        },
        Instruction::Call { ip: 6 },
        Instruction::Ret,
        // Function that increments Y0
        Instruction::Add {
            arg0: Reg::Y(0),
            arg1: Reg::Y(2),
            ret: Reg::Y(0),
        },
        Instruction::IsEq {
            lbl: 9,
            arg0: Reg::Y(0),
            arg1: Reg::Y(1),
        },
        Instruction::Call { ip: 6 },
        Instruction::Ret,
    ];
    // vm.spawn(proc1);
    // thread::sleep(Duration::from_millis(5));
    vm.lock().unwrap().spawn(proc2);
    thread::sleep(Duration::from_millis(500));

    vm.lock().unwrap().wait();
}
