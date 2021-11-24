use crate::heap::class::Class;
use crate::heap::desc::JTypeDescriptor;
use crate::ins::Refs;
use crate::op::OpCode;
use crate::runtime::{frame::JFrame, misc::BytesReader, vm::JThread};

macro_rules! asf {
    ($op: expr, $c: ident, $sym: ident, $mf: ident, $psh: ident, $pp: ident, $t: ty) => {
        match $op {
            putstatic => $c.set_static($sym.member.id, $mf.$pp() as u64),
            getstatic => $mf.$psh($c.get_static($sym.member.id) as $t),
            putfield => {
                let v = $mf.$pp();
                let obj = $mf.pop_obj();
                obj.fields()[$sym.member.id] = v as u64;
            }
            getfield => {
                let obj = $mf.pop_obj();
                let v = $c.get_instance(&obj, $sym.member.id);
                $mf.$psh(v as $t);
            }
            _ => {}
        };
    };
}
impl Refs for OpCode {
    fn refs(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            new => {
                let i = rd.u16() as usize;
                let ptr = {
                    let sym = { mf.class_ref(i) };
                    if sym.class.get_mut().clinit(th) {
                        th.revert_pc();
                        return;
                    }
                    let ptr = Class::new_obj(sym.class);
                    ptr
                };
                mf.push_obj(ptr);
            }
            multianewarray => {
                let a_class = mf.class_ref(rd.u16() as usize);
                let dim = rd.u8() as u16;
                let counts = mf.pop_slots(dim);
                let arr = mf.heap.new_multi_dim(a_class.class, counts);
                mf.push_obj(arr)
            }
            newarray | anewarray => {
                let atype = if self == newarray {
                    rd.u8() as usize
                } else {
                    rd.u16() as usize
                };
                let n = mf.pop_i32() as i32;

                if n < 0 {
                    panic!("java.lang.NegativeArraySize");
                }

                if self == newarray {
                    let arr = mf.heap.new_primitive_array((atype - 4) as i32, n as usize);
                    mf.push_obj(arr);
                } else {
                    let c = mf.class_ref(atype).class;
                    let arr = mf.heap.new_array(&c.name, n as usize);
                    mf.push_obj(arr);
                };
            }
            arraylength => {
                let obj = mf.pop_obj();
                mf.push_u32(obj.size as u32);
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

                if self == invokestatic {
                    if sym.class.get_mut().clinit(th) {
                        th.revert_pc();
                        return;
                    }
                }

                // invoke virtual, resolve method in object class
                if self == invokevirtual || self == invokeinterface {
                    let obj = mf.back_obj(sym.member.m_desc.arg_slots as usize + 1);
                    if obj.is_null() {
                        panic!("java.lang.NullPointerException");
                    }

                    m = obj.class.lookup_method_in_class(&sym.name, &sym.desc);
                }

                let mut new_frame = th.push_frame(m);
                mf.pass_args(
                    &mut new_frame,
                    if self == invokestatic {
                        sym.member.m_desc.arg_slots
                    } else {
                        // +1 this pointer
                        sym.member.m_desc.arg_slots + 1
                    },
                );
            }
            instanceof | checkcast => {
                let i = rd.u16() as usize;
                let sym = mf.class_ref(i);
                let o = mf.pop_obj();

                let is = if o.is_null() {
                    false
                } else {
                    o.instance_of(&sym.class)
                };

                if self == instanceof {
                    mf.push_u32(if is { 1 } else { 0 });
                    return;
                }

                mf.push_obj(o);
            }
            putstatic | getstatic | putfield | getfield => {
                let i = rd.u16() as usize;
                let sym = mf.field_ref(i);

                if self == putstatic || self == getstatic {
                    if sym.class.get_mut().clinit(th) {
                        th.revert_pc();
                        return;
                    }
                }

                let mut class = sym.class;

                match sym.desc.slots() {
                    1 => {
                        asf!(self, class, sym, mf, push_u32, pop_u32, u32);
                    }
                    2 => {
                        asf!(self, class, sym, mf, push_u64, pop_u64, u64);
                    }
                    _ => {
                        asf!(self, class, sym, mf, push_slot, pop_slot, u64);
                    }
                }
            }
            monitorenter | monitorexit => {
                mf.pop_slot();
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
