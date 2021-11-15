use crate::heap::Object;
use crate::ins::Refs;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, vm::JThread, vm::JFrame};
use std::rc::Rc;
use std::cell::RefCell;

impl Refs for OpCode {
    fn refs(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>) {
        use crate::op::OpCode::*;

        match self {
            new => {
                let i = rd.u16() as usize;

                let ptr = {
                    let sym = {
                        frame.borrow().class_ref(i)
                    };

                    let ptr = th.heap.borrow().new_object(sym.class.clone());
                    ptr
                };

                let mut mf = frame.borrow_mut();
                mf.stack.push_cell(Object::forget(ptr));
            },
            invokespecial => {
                rd.u16();
                frame.borrow_mut().stack.pop_cell();
            }
            putstatic | getstatic | putfield | getfield => {
                let i = rd.u16() as usize;
                let sym = frame.borrow().field_ref(i);

                let mut mf = frame.borrow_mut();
                let mut class = sym.class.borrow_mut();

                match sym.desc.as_bytes()[0] {
                    b'Z' | b'B' | b'C' | b'S' | b'I' | b'F' => {
                        match self {
                            putstatic => class.set_static(sym.field_i, mf.stack.pop_u32() as u64),
                            getstatic => mf.stack.push_u32(class.get_static(sym.field_i) as u32),
                            putfield => {
                                let v = mf.stack.pop_u32();
                                let mut obj = Object::from_ptr(mf.stack.pop_cell());
                                class.set_instance(&mut obj, sym.field_i, v as u64);
                                Object::forget(obj);
                            },
                            getfield => {
                                let obj = Object::from_ptr(mf.stack.pop_cell());
                                let v = class.get_instance(&obj, sym.field_i);
                                mf.stack.push_u32(v as u32);
                                Object::forget(obj);
                            }
                            _ => {},
                        };
                    },
                    b'D' | b'J' => {
                        match self {
                            putstatic => class.set_static(sym.field_i, mf.stack.pop_u64() as u64),
                            getstatic => mf.stack.push_u64(class.get_static(sym.field_i)),
                            putfield => {
                                let v = mf.stack.pop_u64();
                                let mut obj = Object::from_ptr(mf.stack.pop_cell());
                                class.set_instance(&mut obj, sym.field_i, v);
                                Object::forget(obj);
                            }
                            getfield => {
                                let obj = Object::from_ptr(mf.stack.pop_cell());
                                let v = class.get_instance(&obj, sym.field_i);
                                mf.stack.push_u64(v);
                                Object::forget(obj);
                            }
                            _ => {},
                        };
                    }
                    b'L' | b'[' => {
                        match self {
                            putstatic => class.set_static(sym.field_i, mf.stack.pop_cell()),
                            getstatic => mf.stack.push_cell(class.get_static(sym.field_i)),
                            putfield => {
                                let v = mf.stack.pop_cell();
                                let mut obj = Object::from_ptr(mf.stack.pop_cell());
                                class.set_instance(&mut obj, sym.field_i, v);
                                Object::forget(obj);
                            }
                            getfield => {
                                let obj = Object::from_ptr(mf.stack.pop_cell());
                                let v = class.get_instance(&obj, sym.field_i);
                                mf.stack.push_cell(v);
                                Object::forget(obj);
                            }
                            _ => {},
                        }
                    }
                    _ => panic!("invalid descriptor {}", sym.desc)
                }
            },
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}