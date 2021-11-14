use crate::heap::Object;
use crate::ins::Refs;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, vm::JThread, vm::JFrame};
use std::rc::Rc;
use std::cell::RefCell;

impl Refs for OpCode {
    fn refs(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>) {
        use crate::op::OpCode::*;
        let mut mf = frame.borrow_mut();

        match self {
            new => {
                // let i = {
                //     rd.u16() as usize
                // };
                // let n = {
                //     mf.class.cp.class(i)
                // };
                // let class = {
                //     let mut heap = th.heap.borrow_mut();
                //     heap.loader.load(n)
                // };

                // if class.access_flags.is_abstract() || class.access_flags.is_iface() {
                //     panic!("java.lang.InstantiationError")
                // }

                // let obj = Object {
                //     class: class.clone(),
                //     fields: vec![0u64; class.ins_fields.len()]
                // };

                // let obj = Box::new(obj);
                // let ptr = Object::forget(obj);
                // mf.stack.push_cell(ptr);
            }

            putstatic => {
                let i = {
                    rd.u16() as usize
                };
                let class = {
                    mf.class.clone()
                };
            }
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}