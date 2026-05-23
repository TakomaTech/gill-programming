use std::fmt;

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Unary { operator: String, right: Box<Expr> },
    Binary { left: Box<Expr>, operator: String, right: Box<Expr> },
    Call { callee: Box<Expr>, arguments: Vec<Expr> },
    Access { object: Box<Expr>, property: String },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expr),
    VariableDeclaration { name: String, value: Expr },
    FunctionDeclaration { name: String, parameters: Vec<String>, body: Vec<Statement> },
    Return(Option<Expr>),
    If { condition: Expr, then_branch: Vec<Statement>, else_branch: Option<Vec<Statement>> },
    While { condition: Expr, body: Vec<Statement> },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub metadata: Vec<(String, String)>,
    pub body: Vec<Statement>,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Number(n) => write!(f, "{}", n),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(value) => write!(f, "{}", value),
            Expr::Identifier(name) => write!(f, "{}", name),
            Expr::Unary { operator, right } => write!(f, "({}{})", operator, right),
            Expr::Binary { left, operator, right } => write!(f, "({} {} {})", left, operator, right),
            Expr::Call { callee, arguments } => {
                let args = arguments.iter().map(|arg| format!("{}", arg)).collect::<Vec<_>>().join(", ");
                write!(f, "{}({})", callee, args)
            }
            Expr::Access { object, property } => write!(f, "{}.{}", object, property),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Expression(expr) => write!(f, "{};", expr),
            Statement::VariableDeclaration { name, value } => write!(f, "local {} = {};", name, value),
            Statement::FunctionDeclaration { name, parameters, body } => {
                let params = parameters.join(", ");
                let body_text = body.iter().map(|stmt| format!("  {}\n", stmt)).collect::<String>();
                write!(f, "func {}({}) {{\n{}}}", name, params, body_text)
            }
            Statement::Return(value) => match value {
                Some(value) => write!(f, "return {};", value),
                None => write!(f, "return;"),
            },
            Statement::If { condition, then_branch, else_branch } => {
                let then_text = then_branch.iter().map(|stmt| format!("  {}\n", stmt)).collect::<String>();
                let else_text = else_branch.as_ref().map(|else_body| {
                    let text = else_body.iter().map(|stmt| format!("  {}\n", stmt)).collect::<String>();
                    format!("else {{\n{}}}", text)
                });
                if let Some(else_text) = else_text {
                    write!(f, "if ({}) {{\n{}}} {}", condition, then_text, else_text)
                } else {
                    write!(f, "if ({}) {{\n{}}}", condition, then_text)
                }
            }
            Statement::While { condition, body } => {
                let body_text = body.iter().map(|stmt| format!("  {}\n", stmt)).collect::<String>();
                write!(f, "while ({}) {{\n{}}}", condition, body_text)
            }
        }
    }
}
