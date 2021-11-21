use crate::heap::class::Class;
use crate::heap::desc::{JType, JTypeDescriptor};
use crate::heap::misc::PRIMITIVES;
use crate::ins::Refs;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, vm::JFrame, vm::JThread};

impl Refs for OpCode {
    fn refs(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            new => {
                let i = rd.u16() as usize;
                let c = mf.method.code[rd.pc as usize];
                let o: OpCode = c.into();
                let ptr = {
                    let sym = { mf.class_ref(i) };
                    if sym.class.get_mut().clinit(th) {
                        th.revert_pc();
                        return;
                    }
                    let ptr = Class::new_obj(sym.class);
                    ptr
                };

                mf.stack.push_obj(ptr);
            }
            multianewarray => {
                let a_class = mf.class_ref(rd.u16() as usize);
                let dim = rd.u8() as usize;
                let counts = &mf.stack.slots[mf.stack.size - dim..mf.stack.size];
                mf.stack.size -= dim;
                let arr = mf.heap.new_multi_dim(a_class.class, counts);
                mf.stack.push_obj(arr)
            }
            newarray | anewarray => {
                let atype = if self == newarray {
                    rd.u8() as usize
                } else {
                    rd.u16() as usize
                };
                let n = mf.stack.pop_i32() as i32;

                if n < 0 {
                    panic!("java.lang.NegativeArraySize");
                }

                if self == newarray {
                    let arr = mf.heap.new_primitive_array((atype - 4) as i32, n as usize);
                    mf.stack.push_obj(arr);
                } else {
                    let c = mf.class_ref(atype).class;

                    let arr = mf.heap.new_array(&c.name, n as usize);
                    mf.stack.push_obj(arr);
                };
            }
            arraylength => {
                let obj = mf.stack.pop_obj();
                mf.stack.push_u32(obj.size as u32);
            }
            invokestatic | invokespecial | invokevirtual | invokeinterface => {
                let sym = if self == invokeinterface {
                    mf.iface_ref(rd.u16() as usize)
                } else {
                    mf.method_ref(rd.u16() as usize)
                };
                if self == invokevirtual && mf.id == 488 {
                    println!("invoke virtual {}.{} ", sym.class.name, sym.member.name);
                }
                if self == invokeinterface {
                    rd.u16();
                }

                let mut m = sym.member;

                // invoke virtual, resolve method in object class
                if self == invokevirtual || self == invokeinterface {
                    let obj = mf.stack.back_obj(sym.member.m_desc.arg_cells + 1);
                    if obj.is_null() {
                        panic!("java.lang.NullPointerException");
                    }

                    if self == invokevirtual && mf.id == 501 {
                        use crate::heap::class::Object;
                        use crate::rp::Rp;
                        let o: Rp<Object> = (mf.stack.slots[0] as usize).into();
                        println!("class of obj = {}", o.class.name);
                        println!("lookup method {} in class {}", m.name, obj.class.name);
                    }
                    m = obj.class.lookup_method_in_class(&sym.name, &sym.desc);
                    if self == invokevirtual && mf.id == 501 {
                        if m.is_null() {
                            println!("method not found");
                        }
                        println!("invoke7");

                        println!("method = {:?}", m.as_ref());
                    }
                }

                let mut new_frame = th.new_frame(m);
                mf.pass_args(
                    &mut new_frame,
                    if self == invokestatic {
                        sym.member.m_desc.arg_cells
                    } else {
                        // +1 this pointer
                        sym.member.m_desc.arg_cells + 1
                    },
                );
                th.push_frame(new_frame);
            }
            instanceof | checkcast => {
                let i = rd.u16() as usize;
                let sym = mf.class_ref(i);
                let o = mf.stack.pop_obj();

                let is = if o.is_null() {
                    false
                } else {
                    o.instance_of(&sym.class)
                };

                if self == instanceof {
                    mf.stack.push_u32(if is { 1 } else { 0 });
                    return;
                }

                if !is {
                    let o = if o.is_null() {
                        "null".to_string()
                    } else {
                        o.class.name.to_string()
                    };
                    // panic!("cannot cast object {} to {}", o, sym.name);
                }

                mf.stack.push_obj(o);
            }
            putstatic | getstatic | putfield | getfield => {
                let i = rd.u16() as usize;
                let sym = mf.field_ref(i);

                let c = mf.method.code[rd.pc as usize];
                if self == putstatic || self == getstatic {
                    if sym.class.get_mut().clinit(th) {
                        th.revert_pc();
                        return;
                    }
                }

                let mut class = sym.class;

                match sym.desc.slots() {
                    1 => {
                        match self {
                            putstatic => class.set_static(sym.member.id, mf.stack.pop_u32() as u64),
                            getstatic => mf.stack.push_u32(class.get_static(sym.member.id) as u32),
                            putfield => {
                                let v = mf.stack.pop_u32();
                                let obj = mf.stack.pop_obj();
                                class.set_instance(obj.get_mut(), sym.member.id, v as u64);
                            }
                            getfield => {
                                let obj = mf.stack.pop_obj();
                                let v = class.get_instance(&obj, sym.member.id);
                                mf.stack.push_u32(v as u32);
                            }
                            _ => {}
                        };
                    }
                    2 => {
                        match self {
                            putstatic => class.set_static(sym.member.id, mf.stack.pop_u64() as u64),
                            getstatic => mf.stack.push_u64(class.get_static(sym.member.id)),
                            putfield => {
                                let v = mf.stack.pop_u64();
                                let obj = mf.stack.pop_obj();
                                class.set_instance(obj.get_mut(), sym.member.id, v);
                            }
                            getfield => {
                                let obj = mf.stack.pop_obj();
                                let v = class.get_instance(&obj, sym.member.id);
                                mf.stack.push_u64(v);
                            }
                            _ => {}
                        };
                    }
                    _ => match self {
                        putstatic => class.set_static(sym.member.id, mf.stack.pop_cell()),
                        getstatic => mf.stack.push_cell(class.get_static(sym.member.id)),
                        putfield => {
                            let v = mf.stack.pop_cell();
                            let obj = mf.stack.pop_obj();
                            class.set_instance(obj.get_mut(), sym.member.id, v);
                        }
                        getfield => {
                            let obj = mf.stack.pop_obj();
                            let v = class.get_instance(&obj, sym.member.id);
                            mf.stack.push_cell(v);
                        }
                        _ => {}
                    },
                }
            }
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    #[test]
    fn test_cow() {
        let abc = "affdsfdsf".to_string();
        let mut cc: Cow<str> = Cow::from(abc);
        modify(cc.to_mut());
        println!("{:?}", cc);
    }

    fn modify(s: &mut str) {
        let m = s.as_mut_ptr();
        unsafe { *m = b'b' }
    }
}
