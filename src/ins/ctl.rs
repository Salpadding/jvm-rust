use crate::ins::Control;
use crate::op::OpCode;
use crate::runtime::{BytesReader, JThread, JFrame};
use std::rc::Rc;
use std::cell::{RefCell, RefMut};

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

    fn exec(&self, th: &mut JThread, rd: &mut BytesReader, mut mf: RefMut<JFrame>) {
        let i = mf.stack.pop_i32();

        let off = 

        if i >= self.low && i <= self.high {
            self.jumps[(i- self.low) as usize] as i32
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

    fn exec(&self, th: &mut JThread, rd: &mut BytesReader, mut mf: RefMut<JFrame>) {
        let k = mf.stack.pop_i32();
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
    fn ctl(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>) {
        use crate::op::OpCode::*;
        use crate::runtime::Slots;
        let mut mf = frame.borrow_mut();

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
                tb.exec(th, rd,  mf);
            }
            lookupswitch => {
                let ls = LookupSwitch::read_from(rd);
                ls.exec(th, rd, mf);
            }
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}