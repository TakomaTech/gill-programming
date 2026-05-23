pub mod ast;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod optimizer;
pub mod parser;

use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::error::GillError;
use crate::interpreter::Interpreter;
use crate::parser::Parser;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Gill parse error: {0}")]
    Parse(#[from] GillError),
}

pub fn run_string(source: &str) -> Result<(), RuntimeError> {
    let mut parser = Parser::new(source);
    let program = parser.parse_program()?;
    let mut interpreter = Interpreter::new();
    interpreter.execute(program)?;
    Ok(())
}

pub fn run_file(path: &Path) -> Result<(), RuntimeError> {
    let source = fs::read_to_string(path)?;
    run_string(&source)
}
