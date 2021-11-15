use crate::ins::Constant;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, vm::JFrame, vm::JThread};

trait Ldc {
    fn _ldc(self, rd:&mut BytesReader, f: &mut JFrame);
}

impl Ldc for OpCode {
    fn _ldc(self, rd:&mut BytesReader, f: &mut JFrame) {
        use crate::op::OpCode::*;
        let i = match self {
            ldc => rd.u8() as usize,
            _ => rd.u16() as usize,
        };

        let (c, w) = f.class.cp.cell(i);

        if (self == ldc || self == ldc_w) && !w {

            f.stack.push_u32(c as u32);
            return;
        }

        if self == ldc2_w && w {
            f.stack.push_u64(c);
            return;
        }

        panic!("java.lang.ClassFormatError");
    }
}

impl Constant for OpCode {
    fn con(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            nop => {}
            aconst_null => mf.stack.push_null(),
            iconst_m1 => mf.stack.push_u32(-1i32 as u32),
            iconst_0 => mf.stack.push_u32(0),
            iconst_1 => mf.stack.push_u32(1),
            iconst_2 => mf.stack.push_u32(2),
            iconst_3 => mf.stack.push_u32(3),
            iconst_4 => mf.stack.push_u32(4),
            iconst_5 => mf.stack.push_u32(5),

            lconst_0 => mf.stack.push_u64(0),
            lconst_1 => mf.stack.push_u64(1),

            fconst_0 => mf.stack.push_f32(0.0f32),
            fconst_1 => mf.stack.push_f32(1.0f32),
            fconst_2 => mf.stack.push_f32(2.0f32),

            dconst_0 => mf.stack.push_f64(0.0f64),
            dconst_1 => mf.stack.push_f64(1.0f64),

            bipush => {
                let i = rd.u8();
                mf.stack.push_u32(i as i8 as i32 as u32);
            }

            sipush => {
                let i = rd.u16();
                mf.stack.push_u32(i as i16 as i32 as u32);
            }

            ldc | ldc_w | ldc2_w => self._ldc(rd, mf),
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}
