use std::{thread, time::Duration};

use ream::{VM, parse_str};

fn main() {
    let vm = VM::new();
    let proc1 = parse_str(
        "{move, {y, 0}, 0}.
{move, {y, 1}, 8001}.
{move, {y, 2}, 1}.
{call, 5}.
{ret}.
{add, {y, 0}, {y, 2}, {y, 0}}.
{is_eq, 8, {y, 0}, {y, 1}}.
{call, 5}.
{ret}.",
    );
    let proc2 = parse_str(
        "{move, {x, 0}, {pid, 0, 1}}.
{move, {x, 1}, 5000}.
{send}.
{move, {x, 0}, 0}.
{move, {x, 2}, 1}.
{call, 7}.
{ret}.
{add, {x, 0}, {x, 2}, {x, 0}}.
{is_eq, 10, {x, 0}, {x, 1}}.
{call, 7}.
{ret}.",
    );
    vm.lock().unwrap().spawn(proc1);
    // thread::sleep(Duration::from_millis(5));
    vm.lock().unwrap().spawn(proc2);
    thread::sleep(Duration::from_millis(500));

    vm.lock().unwrap().wait();
}
