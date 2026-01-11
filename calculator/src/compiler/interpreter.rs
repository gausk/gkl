use crate::Compile;
use crate::ast::{Node, Operator};

struct Eval;

impl Eval {
    pub fn new() -> Self {
        Self
    }

    pub fn eval(&self, expr: &Node) -> i32 {
        match expr {
            Node::Int(n) => *n,
            Node::UnaryExpr { op, child } => {
                let val = self.eval(child);
                match op {
                    Operator::Plus => val,
                    Operator::Minus => -val,
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let left = self.eval(lhs);
                let right = self.eval(rhs);
                match op {
                    Operator::Plus => left + right,
                    Operator::Minus => left - right,
                }
            }
        }
    }
}

pub struct Interpreter;

impl Compile for Interpreter {
    type Output = i32;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let eval = Eval::new();
        let mut out = 0;
        for node in ast {
            out += eval.eval(&node);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter() {
        assert_eq!(Interpreter::from_source("21 + 6").unwrap(), 27);
        assert_eq!(Interpreter::from_source("1 + 2 -3").unwrap(), 0);
    }
}
