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
                    let ptr = mf.new_obj(sym.class);
                    ptr
                };

                mf.stack.push_obj(ptr);
            }
            invokespecial => {
                rd.u16();
                mf.stack.pop_cell();
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

                let mut class = sym.class;

                match sym.desc.as_bytes()[0] {
                    b'Z' | b'B' | b'C' | b'S' | b'I' | b'F' => {
                        match self {
                            putstatic => {
                                class.set_static(sym.member.id as usize, mf.stack.pop_u32() as u64)
                            }
                            getstatic => mf
                                .stack
                                .push_u32(class.get_static(sym.member.id as usize) as u32),
                            putfield => {
                                let v = mf.stack.pop_u32();
                                let obj = mf.stack.pop_obj();
                                class.set_instance(obj.get_mut(), sym.member.id as usize, v as u64);
                            }
                            getfield => {
                                let obj = mf.stack.pop_obj();
                                let v = class.get_instance(&obj, sym.member.id as usize);
                                mf.stack.push_u32(v as u32);
                            }
                            _ => {}
                        };
                    }
                    b'D' | b'J' => {
                        match self {
                            putstatic => {
                                class.set_static(sym.member.id as usize, mf.stack.pop_u64() as u64)
                            }
                            getstatic => {
                                mf.stack.push_u64(class.get_static(sym.member.id as usize))
                            }
                            putfield => {
                                let v = mf.stack.pop_u64();
                                let obj = mf.stack.pop_obj();
                                class.set_instance(obj.get_mut(), sym.member.id as usize, v);
                            }
                            getfield => {
                                let obj = mf.stack.pop_obj();
                                let v = class.get_instance(&obj, sym.member.id as usize);
                                mf.stack.push_u64(v);
                            }
                            _ => {}
                        };
                    }
                    b'L' | b'[' => match self {
                        putstatic => class.set_static(sym.member.id as usize, mf.stack.pop_cell()),
                        getstatic => mf.stack.push_cell(class.get_static(sym.member.id as usize)),
                        putfield => {
                            let v = mf.stack.pop_cell();
                            let obj = mf.stack.pop_obj();
                            class.set_instance(obj.get_mut(), sym.member.id as usize, v);
                        }
                        getfield => {
                            let obj = mf.stack.pop_obj();
                            let v = class.get_instance(&obj, sym.member.id as usize);
                            mf.stack.push_cell(v);
                        }
                        _ => {}
                    },
                    _ => panic!("invalid descriptor {}", sym.desc),
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
