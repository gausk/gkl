// Operation Code
#[derive(Debug, Copy, Clone)]
// VM Operation Code
pub enum OpCode {
    OpConstant(u16), // pointer to constant table
    OpPop,           // pop is needed for execution
    OpAdd,
    OpSub,
    OpPlus,
    OpMinus,
}

pub fn make_op(op: OpCode) -> Vec<u8> {
    match op {
        OpCode::OpConstant(arg) => vec![0x01, (arg >> 8) as u8, (arg & 0xff) as u8],
        OpCode::OpPop => vec![0x02],
        OpCode::OpAdd => vec![0x03],
        OpCode::OpSub => vec![0x04],
        OpCode::OpPlus => vec![0x0A],
        OpCode::OpMinus => vec![0x0B],
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_make_op() {
        assert_eq!(make_op(OpCode::OpConstant(1)), vec![0x01, 0, 1]);
        assert_eq!(make_op(OpCode::OpConstant(257)), vec![0x01, 1, 1]);
        assert_eq!(make_op(OpCode::OpPop), vec![0x02]);
        assert_eq!(make_op(OpCode::OpMinus), vec![0x0B]);
    }
}
