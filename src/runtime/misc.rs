use crate::heap::class::Object;
use rp::Rp;

pub struct BytesReader<'a> {
    pub bytes: &'a [u8],
    pub pc: u32,
}

macro_rules! br_un {
    ($a: ident, $w: expr) => {
        pub fn $a(&mut self) -> $a {
            let p = self.pc as usize;
            let s = &self.bytes[p..p + $w];
            self.pc += $w;
            let mut b = [0u8; $w];
            b.copy_from_slice(s);
            $a::from_be_bytes(b)
        }
    };
}

impl<'a> BytesReader<'a> {
    pub fn u8(&mut self) -> u8 {
        let u = self.bytes[self.pc as usize];
        self.pc += 1;
        u
    }

    br_un!(u16, 2);
    br_un!(u32, 4);

    pub fn i16(&mut self) -> i16 {
        self.u16() as i16
    }

    pub fn i32(&mut self) -> i32 {
        self.u32() as i32
    }

    pub fn skip_padding(&mut self) {
        loop {
            if self.pc % 4 != 0 {
                self.pc += 1;
            } else {
                break;
            }
        }
    }

    pub fn read_i32s(&mut self, n: usize) -> Vec<i32> {
        let mut r: Vec<i32> = Vec::with_capacity(n);
        for _ in 0..n {
            r.push(self.i32());
        }
        r
    }
}
