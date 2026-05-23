use crate::ast::{Expr, Literal, Statement};

pub fn optimize_statements(statements: Vec<Statement>) -> Vec<Statement> {
    statements.into_iter().map(optimize_statement).collect()
}

fn optimize_statement(statement: Statement) -> Statement {
    match statement {
        Statement::Expression(expr) => Statement::Expression(optimize_expr(expr)),
        Statement::VariableDeclaration { name, value } => Statement::VariableDeclaration { name, value: optimize_expr(value) },
        Statement::FunctionDeclaration { name, parameters, body } => Statement::FunctionDeclaration { name, parameters, body: optimize_statements(body) },
        Statement::Return(expr) => Statement::Return(expr.map(optimize_expr)),
        Statement::If { condition, then_branch, else_branch } => Statement::If {
            condition: optimize_expr(condition),
            then_branch: optimize_statements(then_branch),
            else_branch: else_branch.map(optimize_statements),
        },
        Statement::While { condition, body } => Statement::While {
            condition: optimize_expr(condition),
            body: optimize_statements(body),
        },
    }
}

fn optimize_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Literal(_) => expr,
        Expr::Identifier(_) => expr,
        Expr::Unary { operator, right } => {
            let right = optimize_expr(*right);
            if let Expr::Literal(literal) = right {
                if operator == "-" {
                    if let Literal::Number(n) = literal {
                        return Expr::Literal(Literal::Number(-n));
                    }
                }
            }
            Expr::Unary { operator, right: Box::new(right) }
        }
        Expr::Binary { left, operator, right } => {
            let left = optimize_expr(*left);
            let right = optimize_expr(*right);
            match (&left, &right) {
                (Expr::Literal(Literal::Number(a)), Expr::Literal(Literal::Number(b))) => {
                    let value = match operator.as_str() {
                        "+" => Literal::Number(a + b),
                        "-" => Literal::Number(a - b),
                        "*" => Literal::Number(a * b),
                        "/" => Literal::Number(a / b),
                        _ => return Expr::Binary { left: Box::new(left), operator, right: Box::new(right) },
                    };
                    Expr::Literal(value)
                }
                _ => Expr::Binary { left: Box::new(left), operator, right: Box::new(right) },
            }
        }
        Expr::Call { callee, arguments } => {
            let callee = Box::new(optimize_expr(*callee));
            let arguments = arguments.into_iter().map(optimize_expr).collect();
            Expr::Call { callee, arguments }
        }
        Expr::Access { object, property } => Expr::Access { object: Box::new(optimize_expr(*object)), property },
    }
}
