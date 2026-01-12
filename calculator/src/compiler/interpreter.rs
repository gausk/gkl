use crate::Compile;
use crate::ast::{Node, Operator};
use crate::primitive::PrimitiveType;

struct Eval;

impl Eval {
    pub fn new() -> Self {
        Self
    }

    pub fn eval(&self, expr: &Node) -> PrimitiveType {
        match expr {
            Node::Int(n) => (*n).into(),
            Node::Float(f) => (*f).into(),
            Node::UnaryExpr { op, child } => {
                let val = self.eval(child);
                match op {
                    Operator::Plus => val,
                    Operator::Minus => -val,
                    _ => unreachable!(),
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let left = self.eval(lhs);
                let right = self.eval(rhs);
                match op {
                    Operator::Plus => left + right,
                    Operator::Minus => left - right,
                    Operator::Multiply => left * right,
                    Operator::Divide => left / right,
                }
            }
        }
    }
}

pub struct Interpreter;

impl Compile for Interpreter {
    type Output = PrimitiveType;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let eval = Eval::new();
        let mut out = PrimitiveType::Int(0);
        for node in ast {
            out = out + eval.eval(&node);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter() {
        assert_eq!(Interpreter::from_source("21 + 6").unwrap(), 27.into());
        assert_eq!(Interpreter::from_source("1 + 2 -3").unwrap(), 0.into());
    }

    #[test]
    fn test_multiply_and_divide() {
        assert_eq!(Interpreter::from_source("2 * 3").unwrap(), 6.into());
        assert_eq!(Interpreter::from_source("8 / 2").unwrap(), 4.into());
    }

    #[test]
    fn test_operator_precedence() {
        // this should have been 8
        assert_eq!(Interpreter::from_source("2 + 2 * 3").unwrap(), 12.into());
    }

    #[test]
    fn test_float_support() {
        assert_eq!(
            Interpreter::from_source("2.5 + 2.5 + 1.5 + 2").unwrap(),
            8.5.into()
        );
        assert_eq!(Interpreter::from_source("1.2 * 2").unwrap(), 2.4.into());
    }
}
