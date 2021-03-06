use crate::ins::Control;
use crate::op::OpCode;
use crate::runtime::{frame::JFrame, misc::BytesReader, vm::JThread};

#[derive(Debug, Default)]
struct TableSwitch {
    default_off: i32,
    low: i32,
    high: i32,
    jumps: Vec<i32>,
}

impl TableSwitch {
    fn read_from(rd: &mut BytesReader) -> Self {
        rd.skip_padding();
        let mut t: Self = Self::default();
        t.default_off = rd.i32();
        t.low = rd.i32();
        t.high = rd.i32();
        let c = t.high - t.low + 1;
        t.jumps = rd.read_i32s(c as usize);
        t
    }

    fn exec(&self, th: &mut JThread, rd: &mut BytesReader, mf: &mut JFrame) {
        let i = mf.pop_i32();

        let off = if i >= self.low && i <= self.high {
            self.jumps[(i - self.low) as usize] as i32
        } else {
            self.default_off
        };

        th.branch(off)
    }
}

#[derive(Debug, Default)]
struct LookupSwitch {
    default_off: i32,
    n: i32,
    match_offs: Vec<i32>,
}

impl LookupSwitch {
    fn read_from(rd: &mut BytesReader) -> Self {
        rd.skip_padding();
        let mut l = LookupSwitch::default();
        l.default_off = rd.i32();
        l.n = rd.i32();
        l.match_offs = rd.read_i32s((l.n as usize) * 2);
        l
    }

    fn exec(&self, th: &mut JThread, rd: &mut BytesReader, mf: &mut JFrame) {
        let k = mf.pop_i32();
        let mut i = 0i32;

        while i < self.n * 2 {
            if self.match_offs[i as usize] == k {
                th.branch(self.match_offs[(i + 1) as usize]);
                return;
            }

            i += 2;
        }
        th.branch(self.default_off);
    }
}

impl Control for OpCode {
    fn ctl(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            goto => {
                let off = rd.i16() as i32;
                th.branch(off);
            }
            goto_w => {
                let off = rd.i32();
                th.branch(off);
            }
            tableswitch => {
                let tb = TableSwitch::read_from(rd);
                tb.exec(th, rd, mf);
            }
            lookupswitch => {
                let ls = LookupSwitch::read_from(rd);
                ls.exec(th, rd, mf);
            }
            ireturn | lreturn | freturn | dreturn | areturn | return_void => {
                let mut s = self;

                if mf.no_ret {
                    s = return_void;
                }

                if mf.method.name == "equals"
                    && mf.method.desc == "(Ljava/lang/Object;)Z"
                    && mf.class().name == "java/lang/String"
                {
                    println!("compare string {} to other", mf.this().jstring());
                    use crate::heap::class::Object;
                    use rp::Rp;
                    let other: Rp<Object> = (mf.local_vars()[1] as usize).into();
                }

                if s == ireturn || self == freturn {
                    let c = mf.pop_u32();
                    th.prev_frame().push_u32(c)
                }

                if s == lreturn || self == dreturn {
                    let c = mf.pop_u64();
                    th.prev_frame().push_u64(c)
                }

                if s == areturn {
                    let c = mf.pop_slot();
                    if c == 0 {
                        println!("areturn returns a null")
                    }
                    th.prev_frame().push_slot(c)
                }
                // println!(
                //     "exit frame {}.{} id = {}",
                //     mf.method.class.name, mf.method.name, mf.id
                // );
                th.pop_frame();

                if !th.stack().is_empty() {
                    // println!(
                    //     "cur frame = {}.{}",
                    //     th.cur_frame().method.class.name,
                    //     th.cur_frame().method.name
                    // );
                }
            }
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}
