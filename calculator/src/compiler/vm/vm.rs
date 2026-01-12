use crate::Compile;
use crate::ast::Node;
use crate::compiler::vm::bytecode::Bytecode;
use crate::compiler::vm::bytecode::Interpreter as ByteCodeInterpreter;

const STACK_SIZE: usize = 512;

pub struct VM {
    bytecode: Bytecode,
    stack: [Node; STACK_SIZE],
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

    pub fn run(&mut self) {
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
                    match (self.pop(), self.pop()) {
                        (Node::Int(rhs), Node::Int(lhs)) => self.push(Node::Int(lhs + rhs)),
                        _ => panic!("Unknown types to OppAdd"),
                    }
                }
                0x04 => {
                    // OpSub
                    match (self.pop(), self.pop()) {
                        (Node::Int(rhs), Node::Int(lhs)) => self.push(Node::Int(lhs - rhs)),
                        _ => panic!("Unknown types to OppSub"),
                    }
                }
                0x05 => {
                    // OpMul
                    match (self.pop(), self.pop()) {
                        (Node::Int(rhs), Node::Int(lhs)) => self.push(Node::Int(lhs * rhs)),
                        _ => panic!("Unknown types to OppMul"),
                    }
                }
                0x06 => {
                    // OpDiv
                    match (self.pop(), self.pop()) {
                        (Node::Int(rhs), Node::Int(lhs)) => self.push(Node::Int(lhs / rhs)),
                        _ => panic!("Unknown types to OppDiv"),
                    }
                }
                0x0A => {
                    // OpPlus
                    match self.pop() {
                        Node::Int(val) => self.push(Node::Int(val)),
                        _ => panic!("Unknown types to OpPlus"),
                    }
                }
                0x0B => {
                    // OpMinus
                    match self.pop() {
                        Node::Int(val) => self.push(Node::Int(-val)),
                        _ => panic!("Unknown types to OpMinus"),
                    }
                }
                _ => panic!("Invalid instruction"),
            }
        }
    }

    pub fn push(&mut self, node: Node) {
        self.stack[self.stack_ptr] = node;
        self.stack_ptr += 1;
    }

    pub fn pop(&mut self) -> Node {
        self.stack_ptr -= 1;
        self.stack[self.stack_ptr].clone()
    }

    pub fn last_popped(&self) -> &Node {
        // the stack pointer points to the next "free" space
        // which also hold most recently popped element.
        &self.stack[self.stack_ptr]
    }
}

impl Compile for VM {
    type Output = i32;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let mut bytecode = ByteCodeInterpreter::from_ast(ast);
        let mut vm = VM::new(bytecode);
        vm.run();
        match vm.last_popped() {
            Node::Int(val) => *val,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::vm::bytecode::Interpreter;

    #[test]
    fn test_vm() {
        let source = "1 + ((2 + 3) - (2 + 3))";
        let byte_code = Interpreter::from_source(source).unwrap();
        println!("{:?}", byte_code);
        let mut vm = VM::new(byte_code);
        vm.run();
        assert_eq!(vm.last_popped(), &Node::Int(1));
    }

    #[test]
    fn test_multiply() {
        let source = "1 + ((2 * 3) - (6 / 3))";
        let byte_code = Interpreter::from_source(source).unwrap();
        println!("{:?}", byte_code);
        let mut vm = VM::new(byte_code);
        vm.run();
        assert_eq!(vm.last_popped(), &Node::Int(5));
    }
}
