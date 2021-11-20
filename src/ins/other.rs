use crate::ins::Other;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, vm::JFrame, vm::JThread};

impl Other for OpCode {
    fn other(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            impdep1 => {
                let w = th.registry.find(
                    mf.class.name.as_str(),
                    mf.method.name.as_str(),
                    mf.method.desc.as_str(),
                );
                w.inner.exec(mf);
            }
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}
