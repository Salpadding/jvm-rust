use crate::ins::Load;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, vm::JFrame, vm::JThread};

macro_rules! xload {
    ($rd: ident, $mf: ident, $gt: ident, $pt: ident, $wd: expr) => {{
        let i = if $wd {
            $rd.u16() as usize
        } else {
            $rd.u8() as usize
        };
        let v = { $mf.local_vars.$gt(i) };
        $mf.stack.$pt(v);
    }};
}

macro_rules! iload_n {
    ($mf: ident, $n: expr) => {{
        let v = { $mf.local_vars.get_u32($n) };
        $mf.stack.push_u32(v);
    }};
}

macro_rules! aload_n {
    ($mf: ident, $n: expr) => {{
        let v = { $mf.local_vars.get_cell($n) };
        $mf.stack.push_cell(v);
    }};
}

macro_rules! lload_n {
    ($mf: ident, $n: expr) => {{
        let v = { $mf.local_vars.get_u64($n) };
        $mf.stack.push_u64(v);
    }};
}

macro_rules! xaload {
    ($mf: ident, $t: ty, $psh: ident) => {{
        let i = $mf.stack.pop_u32() as usize;
        let obj = $mf.stack.pop_obj();
        let v: $t = obj.get(i);
        $mf.stack.$psh(v);
    }};
}


impl Load for OpCode {
    fn load(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame, w: bool) {
        use crate::op::OpCode::*;
        use crate::runtime::misc::Slots;

        match self {
            iload | fload => xload!(rd, mf, get_u32, push_u32, w),
            lload | dload => xload!(rd, mf, get_u64, push_u64, w),
            aload => xload!(rd, mf, get_cell, push_cell, w),

            iload_0 | fload_0 => iload_n!(mf, 0),
            iload_1 | fload_1 => iload_n!(mf, 1),
            iload_2 | fload_2 => iload_n!(mf, 2),
            iload_3 | fload_3 => iload_n!(mf, 3),

            lload_0 | dload_0 => lload_n!(mf, 0),
            lload_1 | dload_1 => lload_n!(mf, 1),
            lload_2 | dload_2 => lload_n!(mf, 2),
            lload_3 | dload_3 => lload_n!(mf, 3),

            aload_0 => aload_n!(mf, 0),
            aload_1 => aload_n!(mf, 1),
            aload_2 => aload_n!(mf, 2),
            aload_3 => aload_n!(mf, 3),

            iaload | faload | caload => xaload!(mf, u32, push_u32),
            daload | laload => xaload!(mf, u64, push_u64),
            saload => xaload!(mf, u16, push_u16),
            aaload => xaload!(mf, u64, push_cell),
            baload => xaload!(mf, u8, push_u8),
            _ => panic!("invalid op {:?}", self),
        };
    }
}
