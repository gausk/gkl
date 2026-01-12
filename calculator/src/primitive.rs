use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveType {
    Int(i32),
    Float(f64),
}

impl From<i32> for PrimitiveType {
    fn from(n: i32) -> Self {
        PrimitiveType::Int(n)
    }
}

impl From<f64> for PrimitiveType {
    fn from(f: f64) -> Self {
        PrimitiveType::Float(f)
    }
}

impl Neg for PrimitiveType {
    type Output = PrimitiveType;

    fn neg(self) -> Self::Output {
        match self {
            PrimitiveType::Int(n) => PrimitiveType::Int(-n),
            PrimitiveType::Float(f) => PrimitiveType::Float(-f),
        }
    }
}

macro_rules! impl_binary_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for PrimitiveType {
            type Output = PrimitiveType;

            fn $method(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (PrimitiveType::Int(a), PrimitiveType::Int(b)) => PrimitiveType::Int(a $op b ),
                    (PrimitiveType::Int(a), PrimitiveType::Float(b)) => PrimitiveType::Float(a as f64 $op b),
                    (PrimitiveType::Float(a), PrimitiveType::Int(b)) => PrimitiveType::Float(a $op b as f64),
                    (PrimitiveType::Float(a), PrimitiveType::Float(b)) => PrimitiveType::Float(a $op b),
                }
            }
        }
    };
}

impl_binary_op!(Add, add, +);
impl_binary_op!(Sub, sub, -);
impl_binary_op!(Mul, mul, *);
impl_binary_op!(Div, div, /);
