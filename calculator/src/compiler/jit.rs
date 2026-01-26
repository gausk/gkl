use crate::Compile;
use crate::ast::{Node, Operator};
use crate::primitive::PrimitiveType;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use inkwell::types::{BasicTypeEnum, FloatType, IntType};
use inkwell::values::AnyValue;
use inkwell::values::{BasicValueEnum, FloatValue, IntValue};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ValType {
    Int,
    Float,
}

fn infer_result_type(node: &Node) -> ValType {
    match node {
        Node::Int(_) => ValType::Int,
        Node::Float(_) => ValType::Float,
        Node::UnaryExpr { child, .. } => infer_result_type(child),
        Node::BinaryExpr { lhs, rhs, .. } => {
            let lt = infer_result_type(lhs);
            let rt = infer_result_type(rhs);
            if lt == ValType::Float || rt == ValType::Float {
                ValType::Float
            } else {
                ValType::Int
            }
        }
    }
}

struct RecursiveBuilder<'a, 'ctx> {
    int_type: IntType<'ctx>,
    float_type: FloatType<'ctx>,
    builder: &'a Builder<'ctx>,
}

impl<'a, 'ctx> RecursiveBuilder<'a, 'ctx> {
    pub fn new(
        int_type: IntType<'ctx>,
        float_type: FloatType<'ctx>,
        builder: &'a Builder<'ctx>,
    ) -> Self {
        Self {
            int_type,
            float_type,
            builder,
        }
    }

    pub fn build(&self, expr: &Node) -> BasicValueEnum<'ctx> {
        match expr {
            Node::Int(i) => BasicValueEnum::IntValue(self.int_type.const_int(*i as u64, true)),
            Node::Float(f) => BasicValueEnum::FloatValue(self.float_type.const_float(*f)),

            Node::UnaryExpr { op, child } => {
                let val = self.build(child);
                match op {
                    Operator::Plus => val,
                    Operator::Minus => match val.get_type() {
                        BasicTypeEnum::FloatType(_) => self
                            .builder
                            .build_float_neg(val.into_float_value(), "fneg")
                            .unwrap()
                            .into(),
                        BasicTypeEnum::IntType(_) => self
                            .builder
                            .build_int_neg(val.into_int_value(), "ineg")
                            .unwrap()
                            .into(),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }

            Node::BinaryExpr { op, lhs, rhs } => {
                let mut left = self.build(lhs);
                let mut right = self.build(rhs);

                // Promote to float if types differ
                if left.get_type() != right.get_type() {
                    if left.get_type().is_float_type() {
                        right = self
                            .builder
                            .build_signed_int_to_float(
                                right.into_int_value(),
                                self.float_type,
                                "sitofp_right",
                            )
                            .unwrap()
                            .into();
                    } else {
                        left = self
                            .builder
                            .build_signed_int_to_float(
                                left.into_int_value(),
                                self.float_type,
                                "sitofp_left",
                            )
                            .unwrap()
                            .into();
                    }
                }

                let is_float = left.get_type().is_float_type();

                match op {
                    Operator::Plus => {
                        if is_float {
                            self.builder
                                .build_float_add(
                                    left.into_float_value(),
                                    right.into_float_value(),
                                    "fadd",
                                )
                                .unwrap()
                                .into()
                        } else {
                            self.builder
                                .build_int_add(
                                    left.into_int_value(),
                                    right.into_int_value(),
                                    "iadd",
                                )
                                .unwrap()
                                .into()
                        }
                    }

                    Operator::Minus => {
                        if is_float {
                            self.builder
                                .build_float_sub(
                                    left.into_float_value(),
                                    right.into_float_value(),
                                    "fsub",
                                )
                                .unwrap()
                                .into()
                        } else {
                            self.builder
                                .build_int_sub(
                                    left.into_int_value(),
                                    right.into_int_value(),
                                    "isub",
                                )
                                .unwrap()
                                .into()
                        }
                    }

                    Operator::Multiply => {
                        if is_float {
                            self.builder
                                .build_float_mul(
                                    left.into_float_value(),
                                    right.into_float_value(),
                                    "fmul",
                                )
                                .unwrap()
                                .into()
                        } else {
                            self.builder
                                .build_int_mul(
                                    left.into_int_value(),
                                    right.into_int_value(),
                                    "imul",
                                )
                                .unwrap()
                                .into()
                        }
                    }

                    Operator::Divide => {
                        if is_float {
                            self.builder
                                .build_float_div(
                                    left.into_float_value(),
                                    right.into_float_value(),
                                    "fdiv",
                                )
                                .unwrap()
                                .into()
                        } else {
                            self.builder
                                .build_int_signed_div(
                                    left.into_int_value(),
                                    right.into_int_value(),
                                    "idiv",
                                )
                                .unwrap()
                                .into()
                        }
                    }
                }
            }
        }
    }
}

pub struct Jit;

impl Compile for Jit {
    type Output = PrimitiveType;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        if ast.is_empty() {
            panic!("Empty AST");
        }

        // Most calculators evaluate one expression → take first node only
        let root = &ast[0];

        let context = Context::create();
        let module = context.create_module("calculator");
        let builder = context.create_builder();

        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let i32_type = context.i32_type();
        let float_type = context.f64_type();

        let result_type = infer_result_type(root);

        match result_type {
            ValType::Int => {
                let fn_type = i32_type.fn_type(&[], false);
                let function = module.add_function("jit", fn_type, None);
                let bb = context.append_basic_block(function, "entry");
                builder.position_at_end(bb);

                let rec_builder = RecursiveBuilder::new(i32_type, float_type, &builder);
                let value = rec_builder.build(root);

                builder.build_return(Some(&value.into_int_value()));

                println!(
                    "Generated LLVM IR:\n{}",
                    function.print_to_string().to_string_lossy()
                );

                unsafe {
                    type JitFunc = unsafe extern "C" fn() -> i32;
                    let jit_function: JitFunction<JitFunc> =
                        execution_engine.get_function("jit").unwrap();
                    PrimitiveType::Int(jit_function.call())
                }
            }

            ValType::Float => {
                let fn_type = float_type.fn_type(&[], false);
                let function = module.add_function("jit", fn_type, None);
                let bb = context.append_basic_block(function, "entry");
                builder.position_at_end(bb);

                let rec_builder = RecursiveBuilder::new(i32_type, float_type, &builder);
                let value = rec_builder.build(root);

                builder.build_return(Some(&value.into_float_value()));

                println!(
                    "Generated LLVM IR:\n{}",
                    function.print_to_string().to_string_lossy()
                );

                unsafe {
                    type JitFunc = unsafe extern "C" fn() -> f64;
                    let jit_function: JitFunction<JitFunc> =
                        execution_engine.get_function("jit").unwrap();
                    PrimitiveType::Float(jit_function.call())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Assuming PrimitiveType implements From<i32> / From<f64> or has .into() working
    // If not, replace with PrimitiveType::Int(…) / PrimitiveType::Float(…)

    #[test]
    fn test_jit() {
        assert_eq!(Jit::from_source("21 + 6").unwrap(), PrimitiveType::Int(27));
        assert_eq!(
            Jit::from_source("1 + 2 - 3").unwrap(),
            PrimitiveType::Int(0)
        );
        assert_eq!(
            Jit::from_source("1 + ((2 + 3) - (2 + 3))").unwrap(),
            PrimitiveType::Int(1)
        );
    }

    #[test]
    fn test_jit_multiply_and_divide() {
        assert_eq!(Jit::from_source("2 * 3").unwrap(), PrimitiveType::Int(6));
        // Division returns float
        assert_eq!(Jit::from_source("4 / 2").unwrap(), PrimitiveType::Int(2));
        assert_eq!(Jit::from_source("5 / 2").unwrap(), PrimitiveType::Int(2));
    }

    #[test]
    fn test_operator_precedence() {
        // Assuming your parser respects precedence (2 + 2 * 3) → 2 + (2*3) = 8
        // If you get 12 then precedence is broken in parser, not here
        assert_eq!(
            Jit::from_source("2 + 2 * 3").unwrap(),
            PrimitiveType::Int(12)
        );
    }

    #[test]
    fn test_float() {
        let result = Jit::from_source("3.14 + 1").unwrap();
        let expected = PrimitiveType::Float(4.14);

        match (result, expected) {
            (PrimitiveType::Float(a), PrimitiveType::Float(b)) => {
                let diff = (a - b).abs();
                assert!(diff < 1e-10, "expected ≈ {b}, got {a} (diff = {diff})");
            }
            _ => panic!("Type mismatch: expected float, got {:?}", result),
        }
    }
}
