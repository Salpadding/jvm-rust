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
    jumps: Vec<u32>,
}

impl TableSwitch {
    fn read_from(rd: &mut BytesReader) -> Self {
       rd.skip_padding();
       let mut t: Self = Self::default();
       t.default_off = rd.i32();
       t.low = rd.i32();
       t.high = rd.i32();
       let c = t.high - t.low + 1;
       t.jumps = rd.read_u32s(c as usize);
       t
    }

    fn exec(&self, rd: &mut BytesReader, mut mf: RefMut<JFrame>) {
        let i = mf.stack.pop_i32();

        let off = 

        if i >= self.low && i <= self.high {
            self.jumps[(i- self.low) as usize] as i32 as isize
        } else {
            self.default_off as isize
        };

        rd.branch(off) 
    }
}

#[derive(Debug, Default)]
struct LookupSwitch {
    default_off: i32,
    n: i32,
    match_offs: Vec<u32>,
}


impl LookupSwitch {
    fn read_from(rd: &mut BytesReader) -> Self {
        rd.skip_padding();
        let mut l = LookupSwitch::default();
        l.default_off = rd.i32();
        l.n = rd.i32();
        l.match_offs = rd.read_u32s((l.n as usize) * 2);
        l
    }

    fn exec(&self, rd: &mut BytesReader, mut mf: RefMut<JFrame>) {
        let k = mf.stack.pop_u32();
        let mut i = 0i32;

        while i < self.n * 2 {
            if self.match_offs[i as usize] == k {
                rd.branch(self.match_offs[(i + 1) as usize] as i32 as isize);
               return; 
            }

            i += 2; 
        }
        rd.branch(self.default_off as isize);
    }
}

impl Control for OpCode {
    fn ctl(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>) {
        use crate::op::OpCode::*;
        use crate::runtime::Slots;
        let mut mf = frame.borrow_mut();

        match self {
            goto => {
                rd.pc += rd.u16() as usize;
            }
            tableswitch => {
                let tb = TableSwitch::read_from(rd);
                tb.exec(rd,  mf);
            }
            lookupswitch => {
                let ls = LookupSwitch::read_from(rd);
                ls.exec(rd, mf);
            }
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}