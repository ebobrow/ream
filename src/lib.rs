#![feature(mapped_lock_guards)]

pub use instr::Instruction;
pub use mem::{DataObject, PID, stack::Reg};
pub use vm::VM;

mod instr;
mod mem;
mod message;
mod pcb;
mod scheduler;
mod vm;

use std::io::{self, BufRead, Write};

use lrlex::lrlex_mod;
use lrpar::lrpar_mod;

lrlex_mod!("byte.l");
lrpar_mod!("byte.y");

pub use byte_y::{Item, List, Prog};

pub fn repl() {
    let lexerdef = byte_l::lexerdef();
    let stdin = io::stdin();
    loop {
        print!(">>> ");
        io::stdout().flush().ok();
        match stdin.lock().lines().next() {
            Some(Ok(ref l)) => {
                if l.trim().is_empty() {
                    continue;
                }
                // Now we create a lexer with the `lexer` method with which
                // we can lex an input.
                let lexer = lexerdef.lexer(l);
                // Pass the lexer to the parser and lex and parse the input.
                let (res, errs) = byte_y::parse(&lexer);
                for e in errs {
                    println!("{}", e.pp(&lexer, &byte_y::token_epp));
                }
                for list in res.unwrap().ok().unwrap() {
                    println!("{:?}", Instruction::from(&list));
                }
            }
            _ => break,
        }
    }
}

pub fn parse_str(s: &str) -> Vec<Instruction> {
    let lexerdef = byte_l::lexerdef();
    let lexer = lexerdef.lexer(s);
    let (res, errs) = byte_y::parse(&lexer);
    for e in errs {
        println!("{}", e.pp(&lexer, &byte_y::token_epp));
    }
    res.unwrap()
        .ok()
        .unwrap()
        .iter()
        .map(Instruction::from)
        .collect()
}
