mod cmp;
mod cons;
mod conv;
mod ctl;
mod load;
mod math;
mod other;
mod refs;
mod stack;
mod store;

use crate::op::OpCode;
use crate::runtime::{frame::JFrame, misc::BytesReader, vm::JThread};

trait Constant {
    fn con(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

trait Load {
    fn load(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame, wide: bool);
}

trait Store {
    fn store(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame, wide: bool);
}

trait Stack {
    fn stack(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

trait Math {
    fn math(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame, wide: bool);
}

pub trait Conversion {
    fn conv(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

trait Compare {
    fn cmp(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

trait Control {
    fn ctl(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

pub trait Ins {
    fn step(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame, wide: bool);
}

trait Refs {
    fn refs(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

trait Other {
    fn other(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

impl Ins for u8 {
    fn step(self, rd: &mut BytesReader, th: &mut JThread, c: &mut JFrame, wide: bool) {
        let op: OpCode = self.into();

        // if c.method.name != "hashCode" && c.method.name != "equals" {
        //     println!(
        //     "op code = {:?} class = {} method = {} desc = {} native = {} pc = {} stack size = {} frame id = {}",
        //     op,
        //     c.class().name,
        //     c.method.name,
        //     c.method.desc,
        //     c.method.access_flags.is_native(),
        //     rd.pc - 1,
        //     c.stack_size,
        //     c.id,
        // );
        // }

        match self {
            0x00..=0x14 => op.con(rd, th, c),
            0x15..=0x35 => op.load(rd, th, c, wide),
            0x36..=0x56 => op.store(rd, th, c, wide),
            0x57..=0x5f => op.stack(rd, th, c),
            0x60..=0x84 => op.math(rd, th, c, wide),
            0x85..=0x93 => op.conv(rd, th, c),
            // ifnull ifnonnull
            0x94..=0xa6 | 0xc6 | 0xc7 => op.cmp(rd, th, c),
            0xa7..=0xb1 => op.ctl(rd, th, c),
            // multinewarray
            0xb2..=0xc3 | 0xc5 => {
                op.refs(rd, th, c);
            }
            0xca..=0xff => op.other(rd, th, c),

            _ => panic!("invalid op {}", self),
        }
    }
}
