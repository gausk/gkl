use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
}

impl<'a> From<&'a str> for Operator {
    fn from(s: &'a str) -> Self {
        match s {
            "+" => Operator::Plus,
            "-" => Operator::Minus,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Operator::Plus => write!(f, "+"),
            Operator::Minus => write!(f, "-"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Int(i32),
    UnaryExpr {
        op: Operator,
        child: Box<Node>,
    },
    BinaryExpr {
        op: Operator,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Node::Int(n) => write!(f, "{}", n),
            Node::UnaryExpr { op, child } => write!(f, "{}{}", op, child),
            Node::BinaryExpr { op, lhs, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
        }
    }
}
