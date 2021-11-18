use crate::heap::desc::{JType, JTypeDescriptor};
use crate::ins::Refs;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, vm::JFrame, vm::JThread};

impl Refs for OpCode {
    fn refs(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            new => {
                let i = rd.u16() as usize;

                let ptr = {
                    let sym = { mf.class_ref(i) };
                    if sym.class.get_mut().clinit(th) {
                        return;
                    }
                    let ptr = mf.new_obj(sym.class);
                    ptr
                };

                mf.stack.push_obj(ptr);
            }
            newarray | anewarray => {
                let atype = rd.u16() as usize;
                let n = mf.stack.pop_i32() as i32;

                if n < 0 {
                    panic!("java.lang.NegativeArraySize");
                }

                let id = if self == newarray {
                    atype - 4
                } else {
                    mf.class_ref(atype).class.id
                };

                let arr = mf.heap.new_array(id, n as usize);
                mf.stack.push_obj(arr);
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
                if self == invokeinterface {
                    rd.u16();
                }

                let mut m = sym.member;

                // hack System.out.println
                if m.name == "println" {
                    match m.desc.as_str() {
                        "(I)V" => {
                            println!("{}", mf.stack.pop_i32());
                            // pop this pointer
                            mf.stack.pop_cell();
                        }
                        "(J)V" => {
                            println!("{}", mf.stack.pop_i64());
                            // pop this pointer
                            mf.stack.pop_cell();
                        }
                        _ => {}
                    }
                    return;
                }

                // invoke virtual, resolve method in object class
                if self == invokevirtual || self == invokeinterface {
                    let obj = mf.stack.back_obj(sym.member.arg_cells + 1);
                    if obj.is_null() {
                        panic!("java.lang.NullPointerException");
                    }

                    m = obj.class.lookup_method_in_class(&sym.name, &sym.desc);
                }

                let mut new_frame = th.new_frame(m);
                mf.pass_args(
                    &mut new_frame,
                    if self == invokestatic {
                        sym.member.arg_cells
                    } else {
                        // +1 this pointer
                        sym.member.arg_cells + 1
                    },
                );
                th.stack.push_frame(new_frame);
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
                    panic!("cannot cast object {} to {}", o, sym.name);
                }

                mf.stack.push_obj(o);
            }
            putstatic | getstatic | putfield | getfield => {
                let i = rd.u16() as usize;
                let sym = mf.field_ref(i);

                if self == putstatic || self == getstatic {
                    if sym.class.get_mut().clinit(th) {
                        return;
                    }
                }

                let mut class = sym.class;

                match sym.desc.jtype() {
                    JType::IF => {
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
                    JType::DJ => {
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
                    JType::A => match self {
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
