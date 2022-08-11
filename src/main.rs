pub mod chunk;
pub mod debug;
pub mod value;
pub mod vm;
pub mod scanner;
pub mod parser;
pub mod compiler;
pub mod span;
pub mod driver;
pub mod obj;

use std::io;
use std::env;
use std::io::Write;

// use compiler::Compiler;
use driver::Driver;

use crate::vm::InterpretResult;

fn repl() {
    let mut line = String::new();
    loop {
        print!("> ");
        line.clear();
        io::stdout().flush().expect("Fail to flush stdout!");
        io::stdin().read_line(&mut line).expect("Fail to read from stdin!");

        // let mut compiler = Compiler::new(line.clone());
        // compiler.compile();

        if line == ":q\n" {
            break;
        }

        let mut driver = Driver::new();
        driver.debug();

        let res = driver.interpret(line.clone());
        match res {
            InterpretResult::Ok => {},
            _ => { println!("!!!!!! Error: {:?}", res); }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        println!("Hola!");
        println!("I haven't been programmed to compile an entire file. Stay tuned!")
    } else {
        println!("Usage: rlox [path]");
    }
}
