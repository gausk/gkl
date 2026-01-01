use crate::ast::Node;
use crate::ast::Operator;
use anyhow::Result;
use pest::Parser;
use pest::iterators::{Pair, Pairs};

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct CalcParser;

pub fn parse(source: &str) -> Result<Vec<Node>> {
    let pairs = parse_calc(source)?;
    let mut nodes = Vec::new();
    for pair in pairs {
        if let Rule::Expr = pair.as_rule() {
            nodes.push(build_ast_from_expr(pair));
        }
    }
    Ok(nodes)
}

pub fn parse_calc(source: &str) -> Result<Pairs<'_, Rule>> {
    Ok(CalcParser::parse(Rule::Program, source)?)
}

fn build_ast_from_expr(pair: Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::Expr => build_ast_from_expr(pair.into_inner().next().unwrap()),
        Rule::BinaryExpr => {
            let mut pairs = pair.into_inner();

            let first = pairs.next().unwrap();
            // LHS can be UnaryExpr or TERM
            let lhs = match first.as_rule() {
                Rule::UnaryExpr => build_ast_from_unary_expr(first),
                Rule::Term => build_ast_from_term(first),
                _ => unreachable!(),
            };

            let operator = pairs.next().unwrap();
            let op = Operator::from(operator.as_str());
            let rhs = build_ast_from_term(pairs.next().unwrap());
            let mut out = Node::BinaryExpr {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            };
            loop {
                let pair_buf = pairs.next();
                if let Some(pair) = pair_buf {
                    let lhs = out;
                    let op = Operator::from(pair.as_str());
                    let rhs = build_ast_from_expr(pairs.next().unwrap());
                    out = Node::BinaryExpr {
                        lhs: Box::new(lhs),
                        op,
                        rhs: Box::new(rhs),
                    }
                } else {
                    return out;
                }
            }
        }
        Rule::UnaryExpr => build_ast_from_unary_expr(pair),
        other => panic!("Unknown rule for expr: {:?}", other),
    }
}

fn build_ast_from_unary_expr(pair: Pair<Rule>) -> Node {
    let mut pairs = pair.into_inner();
    let operator = pairs.next().unwrap();
    let op = Operator::from(operator.as_str());
    let child = pairs.next().unwrap();
    let child = Box::new(build_ast_from_term(child));
    Node::UnaryExpr { op, child }
}

fn build_ast_from_term(pair: Pair<Rule>) -> Node {
    println!("{:?}", pair.to_string());
    match pair.as_rule() {
        Rule::Int => {
            let int = pair.as_str().parse::<i32>().unwrap();
            Node::Int(int)
        }
        Rule::Expr => build_ast_from_expr(pair),
        other => panic!("unknown term {:?}", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_calc() {
        let source = include_str!("../examples/sample.calc");
        let pairs = parse_calc(source).unwrap();
        assert_eq!(pairs.as_str(), "1 + 2");
        assert_eq!(
            format!("{:?}", pairs),
            include_str!("../test/sample_parser.txt")
        );
    }

    #[test]
    fn test_parse() {
        let nodes = parse("-11 + 2").unwrap();
        assert_eq!(
            nodes,
            vec![Node::BinaryExpr {
                lhs: Box::new(Node::UnaryExpr {
                    op: Operator::Minus,
                    child: Box::new(Node::Int(11))
                }),
                op: Operator::Plus,
                rhs: Box::new(Node::Int(2))
            }]
        )
    }
}
