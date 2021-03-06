use crate::ins::Conversion;
use crate::op::OpCode;
use crate::runtime::{frame::JFrame, misc::BytesReader, vm::JThread};

macro_rules! cv {
    ($mf: ident, $p: ident, $psh: ident, $t: ty) => {{
        let v = $mf.$p();
        let c = v as $t;
        $mf.$psh(c);
    }};
}

macro_rules! i2x {
    ($mf: ident, $t: ty) => {{
        let v = $mf.pop_i32();
        let c = v as $t;
        $mf.push_i32(c as $t as i32);
    }};
}

impl Conversion for OpCode {
    fn conv(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            i2l => cv!(mf, pop_i32, push_i64, i64),
            i2f => cv!(mf, pop_i32, push_f32, f32),
            i2d => cv!(mf, pop_i32, push_f64, f64),
            l2i => cv!(mf, pop_i64, push_i32, i32),
            l2f => cv!(mf, pop_i64, push_f32, f32),
            l2d => cv!(mf, pop_i64, push_f64, f64),
            f2i => cv!(mf, pop_f32, push_i32, i32),
            f2l => cv!(mf, pop_f32, push_i64, i64),
            d2i => cv!(mf, pop_f64, push_i32, i32),
            d2l => cv!(mf, pop_f64, push_i64, i64),
            d2f => cv!(mf, pop_f64, push_f32, f32),
            i2b => i2x!(mf, i8),
            i2c => i2x!(mf, u16),
            i2s => i2x!(mf, i16),
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}
