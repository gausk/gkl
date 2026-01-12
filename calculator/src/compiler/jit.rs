use crate::Compile;
use crate::ast::{Node, Operator};
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use inkwell::types::IntType;
use inkwell::values::{AnyValue, IntValue};

type JitFunc = unsafe extern "C" fn() -> i32;

struct RecursiveBuilder<'a> {
    int_type: IntType<'a>,
    builder: &'a Builder<'a>,
}

impl<'a> RecursiveBuilder<'a> {
    pub fn new(int_type: IntType<'a>, builder: &'a Builder<'a>) -> Self {
        Self { int_type, builder }
    }

    pub fn build(&self, expr: &Node) -> IntValue<'a> {
        match expr {
            Node::Int(i) => self.int_type.const_int(*i as u64, true),
            Node::UnaryExpr { op, child } => {
                let val = self.build(child);
                match op {
                    Operator::Plus => val,
                    Operator::Minus => val.const_neg(),
                    _ => unreachable!(),
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let left = self.build(lhs);
                let right = self.build(rhs);
                match op {
                    Operator::Plus => self.builder.build_int_add(left, right, "add_temp").unwrap(),
                    Operator::Minus => self.builder.build_int_sub(left, right, "sub_temp").unwrap(),
                    Operator::Multiply => {
                        self.builder.build_int_mul(left, right, "mul_temp").unwrap()
                    }
                    Operator::Divide => self
                        .builder
                        .build_int_signed_div(left, right, "div_temp")
                        .unwrap(),
                }
            }
        }
    }
}

pub struct Jit;

impl Compile for Jit {
    type Output = i32;
    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let context = Context::create();
        let module = context.create_module("calculator");

        let builder = context.create_builder();
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let i32_type = context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let function = module.add_function("jit", fn_type, None);
        let basic_block = context.append_basic_block(function, "entry");

        builder.position_at_end(basic_block);

        for node in ast {
            let recursive_builder = RecursiveBuilder::new(i32_type, &builder);
            let out_return = recursive_builder.build(&node);
            let _ = builder.build_return(Some(&out_return));
        }

        println!(
            "Generated LLVM IR: {}",
            function.print_to_string().to_string()
        );

        unsafe {
            let jit_function: JitFunction<JitFunc> = execution_engine.get_function("jit").unwrap();
            jit_function.call()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit() {
        assert_eq!(Jit::from_source("21 + 6").unwrap(), 27);
        assert_eq!(Jit::from_source("1 + 2 -3").unwrap(), 0);
        assert_eq!(Jit::from_source("1 + ((2 + 3) - (2 + 3))").unwrap(), 1);
    }

    #[test]
    fn test_jit_multiply_and_divide() {
        assert_eq!(Jit::from_source("2 * 3").unwrap(), 6);
        assert_eq!(Jit::from_source("4 / 2").unwrap(), 2);
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(Jit::from_source("2 + 2 * 3").unwrap(), 12);
    }
}
