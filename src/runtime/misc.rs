pub(crate) trait DupStack {
    fn dup(&mut self);
    fn dup2(&mut self);
    fn dup_x1(&mut self);
    fn dup_x2(&mut self);
    fn dup2_x1(&mut self);
    fn dup2_x2(&mut self);
    fn swap(&mut self);
}
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

pub trait Slots {
    fn set_u32(&mut self, i: usize, v: u32);
    fn get_u32(&self, i: usize) -> u32;
    fn set_i32(&mut self, i: usize, v: i32);
    fn get_i32(&self, i: usize) -> i32;

    #[inline]
    fn get_f32(&self, i: usize) -> f32 {
        f32::from_bits(self.get_u32(i))
    }

    fn set_u64(&mut self, i: usize, v: u64);

    fn get_u64(&self, i: usize) -> u64;

    #[inline]
    fn get_i64(&self, i: usize) -> i64 {
        self.get_u64(i) as i64
    }

    #[inline]
    fn set_f32(&mut self, i: usize, v: f32) {
        self.set_u32(i, v.to_bits());
    }

    #[inline]
    fn set_f64(&mut self, i: usize, v: f64) {
        self.set_u64(i, v.to_bits());
    }

    #[inline]
    fn get_f64(&self, i: usize) -> f64 {
        f64::from_bits(self.get_u64(i))
    }

    fn get_slot(&self, i: usize) -> u64;
    fn set_slot(&mut self, i: usize, v: u64);
}

impl Slots for [u64] {
    #[inline]
    fn set_u32(&mut self, i: usize, v: u32) {
        self[i] = v as u64;
    }

    #[inline]
    fn get_u32(&self, i: usize) -> u32 {
        self[i] as u32
    }

    #[inline]
    fn set_i32(&mut self, i: usize, v: i32) {
        self[i] = v as u32 as u64;
    }

    #[inline]
    fn get_i32(&self, i: usize) -> i32 {
        self[i] as u32 as i32
    }

    #[inline]
    fn get_slot(&self, i: usize) -> u64 {
        self[i]
    }

    #[inline]
    fn set_slot(&mut self, i: usize, v: u64) {
        self[i] = v;
    }

    #[inline]
    fn set_u64(&mut self, i: usize, v: u64) {
        self[i] = v;
    }

    #[inline]
    fn get_u64(&self, i: usize) -> u64 {
        self[i]
    }
}
