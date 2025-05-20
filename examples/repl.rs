use std::{thread, time::Duration};

use ream::{VM, parse_str};

fn main() {
    let vm = VM::new();
    let proc1 = parse_str(
        "{move, {y, 0}, 0}.
{move, {y, 1}, 8001}.
{move, {y, 2}, 1}.
{call, 1}.
{ret}.
{label, 1}.
{add, {y, 0}, {y, 2}, {y, 0}}.
{is_eq, 2, {y, 0}, {y, 1}}.
{call, 1}.
{label, 2}.
{ret}.",
    );
    let proc2 = parse_str(
        "{move, {x, 0}, {pid, 0, 1}}.
{move, {x, 1}, 5000}.
{send}.
{move, {x, 0}, 0}.
{move, {x, 2}, 1}.
{call, 1}.
{ret}.
{label, 1}.
{add, {x, 0}, {x, 2}, {x, 0}}.
{is_eq, 2, {x, 0}, {x, 1}}.
{call, 1}.
{label, 2}.
{ret}.",
    );
    vm.lock().unwrap().spawn(proc1);
    // thread::sleep(Duration::from_millis(5));
    vm.lock().unwrap().spawn(proc2);
    thread::sleep(Duration::from_millis(500));

    vm.lock().unwrap().wait();
}
