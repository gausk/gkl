use crate::Compile;
use crate::ast::Node;
use crate::ast::Operator;
use crate::compiler::vm::opcode::{OpCode, make_op};
use crate::primitive::PrimitiveType;
use anyhow::Result;
use std::str::RSplit;

#[derive(Debug, Clone, PartialEq)]
pub struct Bytecode {
    pub instructions: Vec<u8>,
    pub constants: Vec<PrimitiveType>,
}

impl Bytecode {
    pub fn new() -> Bytecode {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }
}

pub struct Interpreter {
    pub bytecode: Bytecode,
}

impl Compile for Interpreter {
    type Output = Result<Bytecode>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let mut interpreter = Interpreter::new();
        for node in ast {
            interpreter.interpret_node(node);
            // pop one element from stack after each expression
            // statement to clean up.
            interpreter.add_instruction(OpCode::OpPop);
        }
        Ok(interpreter.bytecode)
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
        }
    }

    pub fn add_instruction(&mut self, opcode: OpCode) {
        self.bytecode.instructions.extend(make_op(opcode));
    }

    pub fn add_constant(&mut self, node: PrimitiveType) -> u16 {
        self.bytecode.constants.push(node);
        self.bytecode.constants.len() as u16 - 1
    }

    pub fn interpret_node(&mut self, expr: Node) {
        match expr {
            Node::Int(d) => {
                let const_index = self.add_constant(PrimitiveType::Int(d));
                self.add_instruction(OpCode::OpConstant(const_index));
            }
            Node::Float(d) => {
                let const_index = self.add_constant(PrimitiveType::Float(d));
                self.add_instruction(OpCode::OpConstant(const_index));
            }
            Node::UnaryExpr { op, child } => {
                self.interpret_node(*child);
                match op {
                    Operator::Plus => self.add_instruction(OpCode::OpPlus),
                    Operator::Minus => self.add_instruction(OpCode::OpMinus),
                    _ => unreachable!(),
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                self.interpret_node(*lhs);
                self.interpret_node(*rhs);
                match op {
                    Operator::Plus => self.add_instruction(OpCode::OpAdd),
                    Operator::Minus => self.add_instruction(OpCode::OpSub),
                    Operator::Multiply => self.add_instruction(OpCode::OpMul),
                    Operator::Divide => self.add_instruction(OpCode::OpDiv),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::PrimitiveType;

    #[test]
    fn test_interpreter() {
        for sign in ["+", "-"] {
            let input = format!("1 {} 2", sign);
            let bytecode = Interpreter::from_source(&input).unwrap().unwrap();
            let op_code = match sign {
                "+" => OpCode::OpAdd,
                "-" => OpCode::OpSub,
                _ => unreachable!(),
            };
            let expected_instructions = vec![
                OpCode::OpConstant(0),
                OpCode::OpConstant(1),
                op_code,
                OpCode::OpPop,
            ]
            .into_iter()
            .flat_map(|op_code| make_op(op_code))
            .collect();
            assert_eq!(
                Bytecode {
                    instructions: expected_instructions,
                    constants: vec![PrimitiveType::Int(1), PrimitiveType::Int(2)]
                },
                bytecode
            );
        }
    }
}
