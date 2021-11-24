use crate::ins::Stack;
use crate::op::OpCode;
use crate::runtime::{frame::JFrame, misc::BytesReader, misc::DupStack, vm::JThread};

impl Stack for OpCode {
    fn stack(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            pop => mf.drop(1),
            pop2 => mf.drop(2),
            dup => mf.dup(),
            dup_x1 => mf.dup_x1(),
            dup_x2 => mf.dup_x2(),
            dup2 => mf.dup2(),
            dup2_x1 => mf.dup2_x1(),
            dup2_x2 => mf.dup2_x2(),
            swap => mf.swap(),
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}
