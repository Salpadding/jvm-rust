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

// Each frame (§2.6) contains an array of variables known as its local variables. The length of the local variable array of a frame is determined at compile-time and supplied in the binary representation of a class or interface along with the code for the method associated with the frame (§4.7.3).
// A single local variable can hold a value of type boolean, byte, char, short, int, float, reference, or returnAddress. A pair of local variables can hold a value of type long or double.
// Local variables are addressed by indexing. The index of the first local variable is zero. An integer is considered to be an index into the local variable array if and only if that integer is between zero and one less than the size of the local variable array.
// A value of type long or type double occupies two consecutive local variables. Such a value may only be addressed using the lesser index. For example, a value of type double stored in the local variable array at index n actually occupies the local variables with indices n and n+1; however, the local variable at index n+1 cannot be loaded from. It can be stored into. However, doing so invalidates the contents of local variable n.
// The Java Virtual Machine does not require n to be even. In intuitive terms, values of types long and double need not be 64-bit aligned in the local variables array. Implementors are free to decide the appropriate way to represent such values using the two local variables reserved for the value.
// The Java Virtual Machine uses local variables to pass parameters on method invocation. On class method invocation, any parameters are passed in consecutive local variables starting from local variable 0. On instance m
pub trait Slots {
    fn set_u32(&mut self, i: usize, v: u32);
    fn get_u32(&self, i: usize) -> u32;
    fn set_i32(&mut self, i: usize, v: i32);
    fn get_i32(&self, i: usize) -> i32;

    #[inline]
    fn get_f32(&self, i: usize) -> f32 {
        f32::from_bits(self.get_u32(i))
    }

    #[inline]
    fn set_u64(&mut self, i: usize, v: u64) {
        self.set_u32(i, v as u32);
        self.set_u32(i + 1, (v >> 32) as u32);
    }

    #[inline]
    fn get_u64(&self, i: usize) -> u64 {
        let low = self.get_u32(i);
        let high = self.get_u32(i + 1);
        ((high as u64) << 32) | (low as u64)
    }

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
}
