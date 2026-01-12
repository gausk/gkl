#![allow(unused)]
use crate::ast::Node;
use anyhow::Result;

mod ast;
pub mod compiler;
mod parser;
mod primitive;

pub trait Compile {
    type Output;

    fn from_ast(ast: Vec<Node>) -> Self::Output;

    fn from_source(source: &str) -> Result<Self::Output> {
        let ast = parser::parse(source)?;
        Ok(Self::from_ast(ast))
    }
}
