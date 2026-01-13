use crate::Compile;
use crate::ast::Node;
use crate::compiler::vm::bytecode::Bytecode;
use crate::compiler::vm::bytecode::Interpreter as ByteCodeInterpreter;
use crate::primitive::PrimitiveType;
use anyhow::{Result, bail};

const STACK_SIZE: usize = 512;

pub struct VM {
    bytecode: Bytecode,
    stack: [PrimitiveType; STACK_SIZE],
    stack_ptr: usize,
}

fn usize_from_two_u8s(p1: u8, p2: u8) -> usize {
    u16::from_be_bytes([p1, p2]) as usize
}

impl VM {
    pub fn new(bytecode: Bytecode) -> VM {
        Self {
            bytecode,
            stack: unsafe { std::mem::zeroed() },
            stack_ptr: 0,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut ip = 0;
        while ip < self.bytecode.instructions.len() {
            let inst_addr = ip;
            ip += 1;

            match self.bytecode.instructions[inst_addr] {
                0x01 => {
                    // OpConst
                    let const_idx = usize_from_two_u8s(
                        self.bytecode.instructions[ip],
                        self.bytecode.instructions[ip + 1],
                    );
                    ip += 2;
                    self.push(self.bytecode.constants[const_idx].clone());
                }
                0x02 => {
                    // OpPop
                    self.pop();
                }
                0x03 => {
                    // OpAdd
                    let rhs = self.pop();
                    let lhs = self.pop();
                    let value = lhs + rhs;
                    self.push(value);
                }
                0x04 => {
                    // OpSub
                    let rhs = self.pop();
                    let lhs = self.pop();
                    let value = lhs - rhs;
                    self.push(value);
                }
                0x05 => {
                    // OpMul
                    let rhs = self.pop();
                    let lhs = self.pop();
                    let value = lhs * rhs;
                    self.push(value);
                }
                0x06 => {
                    // OpDiv
                    let rhs = self.pop();
                    let lhs = self.pop();
                    let value = lhs / rhs;
                    self.push(value);
                }
                0x0A => {
                    // OpPlus
                    let value = self.pop();
                    self.push(value);
                }
                0x0B => {
                    // OpMinus
                    let value = self.pop();
                    self.push(-value);
                }
                other => bail!("Unknown instruction {}", other),
            }
        }
        Ok(())
    }

    pub fn push(&mut self, node: PrimitiveType) {
        self.stack[self.stack_ptr] = node;
        self.stack_ptr += 1;
    }

    pub fn pop(&mut self) -> PrimitiveType {
        self.stack_ptr -= 1;
        self.stack[self.stack_ptr].clone()
    }

    pub fn last_popped(&self) -> &PrimitiveType {
        // the stack pointer points to the next "free" space
        // which also hold most recently popped element.
        &self.stack[self.stack_ptr]
    }
}

impl Compile for VM {
    type Output = Result<PrimitiveType>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let mut bytecode = ByteCodeInterpreter::from_ast(ast)?;
        let mut vm = VM::new(bytecode);
        vm.run()?;
        Ok(vm.last_popped().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::vm::bytecode::Interpreter;
    use crate::primitive::PrimitiveType;

    #[test]
    fn test_vm() {
        let source = "1 + ((2 + 3) - (2 + 3))";
        let byte_code = Interpreter::from_source(source).unwrap().unwrap();
        println!("{:?}", byte_code);
        let mut vm = VM::new(byte_code);
        vm.run();
        assert_eq!(*vm.last_popped(), 1.into());
    }

    #[test]
    fn test_multiply() {
        let source = "1 + ((2 * 3) - (6 / 3))";
        let byte_code = Interpreter::from_source(source).unwrap().unwrap();
        println!("{:?}", byte_code);
        let mut vm = VM::new(byte_code);
        vm.run();
        assert_eq!(*vm.last_popped(), 5.into());
    }

    #[test]
    fn test_float() {
        let source = "1.2 + 3.6";
        let byte_code = Interpreter::from_source(source).unwrap().unwrap();
        println!("{:?}", byte_code);
        let mut vm = VM::new(byte_code);
        vm.run();
        assert_eq!(*vm.last_popped(), 4.8.into());
    }
}
