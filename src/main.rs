use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;
pub mod environment;
mod interpreter;
pub mod parser;
mod scanner;
pub mod token;
use log::LevelFilter;
use simple_logger::SimpleLogger;

use crate::environment::Environment;
use crate::interpreter::interpret;

struct LoxError {
    exit_code: i32,
}

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .init()
        .unwrap();
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: jlox [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(path_str: &str) -> () {
    let path = Path::new(path_str);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path_str, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    let env = Rc::new(RefCell::new(Environment::new(None)));
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => match run(&s, env) {
            Ok(_) => (),
            Err(err) => std::process::exit(err.exit_code),
        },
    }
}

fn run_prompt() -> () {
    let mut line = 1;
    let env = Rc::new(RefCell::new(Environment::new(None)));
    loop {
        print!("[{}] ", line);
        match std::io::stdout().flush() {
            Ok(_) => {}
            Err(_) => panic!("flushing stdout resulted in an error, aborting"),
        }
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => {
                break;
            }
            Ok(_) => match run(&input, env.clone()) {
                Ok(_) => {
                    line += 1;
                }
                Err(_) => line += 1,
            },
            Err(error) => println!("error: {}", error),
        }
    }
}

fn run(source: &str, env: Rc<RefCell<Environment>>) -> Result<(), LoxError> {
    match scanner::scan_tokens(source) {
        Ok(tokens) => {
            // for token in &tokens[..] {
            //     println!("{:?}", token);
            // }
            match parser::parse(&tokens[..]) {
                Ok(stmts) => match interpret(&stmts, env) {
                    Ok(()) => return Ok(()),
                    Err(errs) => {
                        for interpreter::RuntimeError {
                            expr,
                            line,
                            message,
                        } in errs
                        {
                            println!("{} [line {}]: {}", message, line, expr);
                        }
                        return Err(LoxError { exit_code: 70 });
                    }
                },
                Err(errs) => {
                    for parser::ParseError { token, message } in errs {
                        match token {
                            Some(token) => error(
                                token.line,
                                format!("parser error on {:?}: {}", token, message),
                            ),
                            None => error(0, format!("parser error on {:?}: {}", token, message)),
                        }
                    }
                    return Err(LoxError { exit_code: 65 });
                }
            }
        }
        Err(scanner::ScanError {
            cause,
            line,
            position,
        }) => {
            match cause {
                scanner::ScanErrorType::BadChar(c) => {
                    error(line, format!("Unexpected character {} at {}", c, position))
                }
                scanner::ScanErrorType::UnterminatedString(s) => {
                    error(line, format!("Unterminated string {} at {}", s, position))
                }
                scanner::ScanErrorType::NumberParseError(s, e) => error(
                    line,
                    format!("Could not parse {} as a number at {} ({})", s, position, e),
                ),
            }
            return Err(LoxError { exit_code: 65 });
        }
    }
}

pub fn error(line: u32, message: String) -> () {
    report(line, "", message);
}

pub fn report(line: u32, location: &str, message: String) -> () {
    println!("[line {}] Error {}: {}", line, location, message);
}
