use crate::ins::Store;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, misc::Slots, vm::JFrame, vm::JThread};

macro_rules! xstore {
    ($rd: ident, $mf: ident, $p: ident, $s: ident, $w: expr) => {{
        let end = if $w {
            $rd.u16() as usize
        } else {
            $rd.u8() as usize
        };
        let v = { $mf.stack.$p() };
        $mf.local_vars.$s(end, v);
    }};
}

macro_rules! istore_n {
    ($mf: ident, $n: expr) => {{
        let v = { $mf.stack.pop_u32() };
        $mf.local_vars.set_u32($n, v);
    }};
}

macro_rules! lstore_n {
    ($mf: ident, $n: expr) => {{
        let v = { $mf.stack.pop_u64() };
        $mf.local_vars.set_u64($n, v);
    }};
}

macro_rules! astore_n {
    ($mf: ident, $n: expr) => {{
        let v = { $mf.stack.pop_cell() };
        $mf.local_vars.set_cell($n, v);
    }};
}

macro_rules! xastore {
    ($mf: ident, $p: ident) => {{
        let v = $mf.stack.$p();
        let i = $mf.stack.pop_u32() as usize;
        let mut obj = $mf.stack.pop_obj();
        obj.set(i, v);
    }};
}

impl Store for OpCode {
    fn store(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame, w: bool) {
        use crate::op::OpCode::*;

        match self {
            istore | fstore => xstore!(rd, mf, pop_u32, set_u32, w),
            lstore | dstore => xstore!(rd, mf, pop_u64, set_u64, w),
            astore => xstore!(rd, mf, pop_cell, set_cell, w),
            istore_0 | fstore_0 => istore_n!(mf, 0),
            istore_1 | fstore_1 => istore_n!(mf, 1),
            istore_2 | fstore_2 => istore_n!(mf, 2),
            istore_3 | fstore_3 => istore_n!(mf, 3),
            lstore_0 | dstore_0 => lstore_n!(mf, 0),
            lstore_1 | dstore_1 => lstore_n!(mf, 1),
            lstore_2 | dstore_2 => lstore_n!(mf, 2),
            lstore_3 | dstore_3 => lstore_n!(mf, 3),
            astore_0 => astore_n!(mf, 0),
            astore_1 => astore_n!(mf, 1),
            astore_2 => astore_n!(mf, 2),
            astore_3 => astore_n!(mf, 3),
            iastore | fastore | castore => xastore!(mf, pop_u32),
            dastore | lastore => xastore!(mf, pop_u64),
            sastore => xastore!(mf, pop_u16),
            aastore => xastore!(mf, pop_cell),
            bastore => xastore!(mf, pop_u8),
            _ => panic!("invalid op {:?}", self),
        };
    }
}
