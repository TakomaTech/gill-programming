use std::collections::HashMap;

use crate::ast::{Expr, Literal, Program, Statement};
use crate::error::GillError;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    Function { parameters: Vec<String>, body: Vec<Statement>, environment: Environment },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(num) => write!(f, "{}", num),
            Value::String(text) => write!(f, "{}", text),
            Value::Boolean(value) => write!(f, "{}", value),
            Value::Nil => write!(f, "nil"),
            Value::Function { parameters, .. } => write!(f, "<fn({})>", parameters.join(", ")),
        }
    }
}

#[derive(Debug, Clone)]n
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self { values: HashMap::new(), enclosing: None }
    }

    pub fn with_enclosing(enclosing: Environment) -> Self {
        Self { values: HashMap::new(), enclosing: Some(Box::new(enclosing)) }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            return Some(value.clone());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name);
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Option<Value> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value.clone());
            return Some(value);
        }
        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.assign(name, value);
        }
        None
    }
}

#[derive(Debug)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self { environment: Environment::new() };
        interpreter.setup_builtins();
        interpreter
    }

    fn setup_builtins(&mut self) {
        self.environment.define("print.console".into(), Value::Function {
            parameters: vec!["value".into()],
            body: vec![Statement::Expression(Expr::Call { callee: Box::new(Expr::Identifier("__builtin_print_console".into())), arguments: vec![Expr::Identifier("value".into())] })],
            environment: self.environment.clone(),
        });
    }

    pub fn execute(&mut self, program: Program) -> Result<Value, GillError> {
        for statement in program.body {
            if let Statement::FunctionDeclaration { name, parameters, body } = statement.clone() {
                self.environment.define(name.clone(), Value::Function { parameters, body, environment: self.environment.clone() });
            }
        }
        if let Some(main_fn) = self.environment.get("main") {
            self.call_function(main_fn, vec![])
        } else {
            self.execute_block(program.body, &mut Environment::with_enclosing(self.environment.clone()))
        }
    }

    fn execute_block(&mut self, statements: Vec<Statement>, environment: &mut Environment) -> Result<Value, GillError> {
        let previous = std::mem::replace(&mut self.environment, environment.clone());
        self.environment = environment.clone();
        let mut last_value = Value::Nil;
        for statement in statements {
            last_value = self.execute_statement(statement)?;
        }
        let restored = std::mem::replace(&mut self.environment, previous);
        *environment = restored;
        Ok(last_value)
    }

    fn execute_statement(&mut self, statement: Statement) -> Result<Value, GillError> {
        match statement {
            Statement::Expression(expr) => self.evaluate(expr),
            Statement::VariableDeclaration { name, value } => {
                let value = self.evaluate(value)?;
                self.environment.define(name, value);
                Ok(Value::Nil)
            }
            Statement::FunctionDeclaration { name, parameters, body } => {
                self.environment.define(name, Value::Function { parameters, body, environment: self.environment.clone() });
                Ok(Value::Nil)
            }
            Statement::Return(expr) => match expr {
                Some(value) => self.evaluate(value),
                None => Ok(Value::Nil),
            },
            Statement::If { condition, then_branch, else_branch } => {
                let condition = self.evaluate(condition)?;
                if self.is_truthy(&condition) {
                    self.execute_block(then_branch, &mut Environment::with_enclosing(self.environment.clone()))
                } else if let Some(else_branch) = else_branch {
                    self.execute_block(else_branch, &mut Environment::with_enclosing(self.environment.clone()))
                } else {
                    Ok(Value::Nil)
                }
            }
            Statement::While { condition, body } => {
                while self.is_truthy(&self.evaluate(condition.clone())?) {
                    self.execute_block(body.clone(), &mut Environment::with_enclosing(self.environment.clone()))?;
                }
                Ok(Value::Nil)
            }
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Value, GillError> {
        match expr {
            Expr::Literal(literal) => Ok(self.evaluate_literal(literal)),
            Expr::Identifier(name) => self.environment.get(&name).ok_or_else(|| GillError::RuntimeError(format!("Undefined identifier: {name}"))),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(*right)?;
                self.evaluate_unary(operator, right)
            }
            Expr::Binary { left, operator, right } => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;
                self.evaluate_binary(operator, left, right)
            }
            Expr::Call { callee, arguments } => {
                let callee = self.evaluate(*callee)?;
                let values = arguments.into_iter().map(|arg| self.evaluate(arg)).collect::<Result<Vec<_>, _>>()?;
                self.call_function(callee, values)
            }
            Expr::Access { object, property } => {
                let object_val = self.evaluate(*object)?;
                match object_val {
                    Value::String(text) if property == "length" => Ok(Value::Number(text.len() as f64)),
                    _ => Err(GillError::RuntimeError(format!("Unknown access property: {property}"))),
                }
            }
        }
    }

    fn call_function(&mut self, callee: Value, arguments: Vec<Value>) -> Result<Value, GillError> {
        match callee {
            Value::Function { parameters, body, environment } => {
                let mut local_env = Environment::with_enclosing(environment);
                for (index, name) in parameters.iter().enumerate() {
                    let value = arguments.get(index).cloned().unwrap_or(Value::Nil);
                    local_env.define(name.clone(), value);
                }
                let result = self.execute_block(body, &mut local_env);
                result
            }
            Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Nil => Err(GillError::RuntimeError("Attempted to call non-function value".into())),
        }
    }

    fn evaluate_literal(&self, literal: Literal) -> Value {
        match literal {
            Literal::Number(value) => Value::Number(value),
            Literal::String(value) => Value::String(value),
            Literal::Boolean(value) => Value::Boolean(value),
            Literal::Nil => Value::Nil,
        }
    }

    fn evaluate_unary(&self, operator: String, right: Value) -> Result<Value, GillError> {
        match operator.as_str() {
            "-" => match right {
                Value::Number(value) => Ok(Value::Number(-value)),
                _ => Err(GillError::RuntimeError("Unary '-' applied to non-number".into())),
            },
            "!" => Ok(Value::Boolean(!self.is_truthy(&right))),
            _ => Err(GillError::RuntimeError(format!("Unknown unary operator: {operator}"))),
        }
    }

    fn evaluate_binary(&self, operator: String, left: Value, right: Value) -> Result<Value, GillError> {
        match operator.as_str() {
            "+" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
                (Value::String(a), Value::Number(b)) => Ok(Value::String(a + &b.to_string())),
                _ => Err(GillError::RuntimeError("Invalid operands for '+'".into())),
            },
            "-" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                _ => Err(GillError::RuntimeError("Invalid operands for '-'".into())),
            },
            "*" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                _ => Err(GillError::RuntimeError("Invalid operands for '*'".into())),
            },
            "/" => match (left, right) {
                (Value::Number(a), Value::Number(b)) if b != 0.0 => Ok(Value::Number(a / b)),
                _ => Err(GillError::RuntimeError("Invalid operands or divide by zero".into())),
            },
            ">" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
                _ => Err(GillError::RuntimeError("Invalid operands for '>'".into())),
            },
            "<" => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
                _ => Err(GillError::RuntimeError("Invalid operands for '<'".into())),
            },
            "==" => Ok(Value::Boolean(left.to_string() == right.to_string())),
            "!=" => Ok(Value::Boolean(left.to_string() != right.to_string())),
            "=" => self.environment.assign(&left.to_string(), right.clone()).ok_or_else(|| GillError::RuntimeError(format!("Undefined assignment target: {}", left))) ,
            _ => Err(GillError::RuntimeError(format!("Unknown binary operator: {operator}"))),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(value) => *value,
            Value::Nil => false,
            Value::Number(number) => *number != 0.0,
            Value::String(text) => !text.is_empty(),
            Value::Function { .. } => true,
        }
    }
}
