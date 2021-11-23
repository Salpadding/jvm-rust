use crate::ins::Other;
use crate::op::OpCode;
use crate::runtime::{frame::JFrame, misc::BytesReader, vm::JThread};
use rp::Rp;

impl Other for OpCode {
    fn other(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            impdep1 => {
                let p: Rp<JThread> = th.into();
                let w = th
                    .registry
                    .find(&mf.class().name, &mf.method.name, &mf.method.desc);
                w.inner.exec(p.get_mut(), mf);
            }
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}
