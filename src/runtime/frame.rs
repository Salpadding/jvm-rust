use crate::heap::class::{Class, ClassMember, Object};
use crate::heap::misc::{Heap, SymRef};
use crate::runtime::misc::Slots;
use cp::ConstantPool;
use rp::Rp;

const MAX_JSTACK_SIZE: usize = 1024;
pub struct JStack {
    frames: [JFrame; MAX_JSTACK_SIZE],
    // default stack size = 64k = 64 * 1024
    stack_data: Vec<u64>,
    size: usize,
}

impl JStack {
    pub fn new(heap: Rp<Heap>) -> Self {
        let mut r = Self {
            frames: [JFrame::default(); MAX_JSTACK_SIZE],
            size: 0,
            stack_data: vec![0; 64 * 1024 / 8],
        };

        for i in 0..r.frames.len() {
            r.frames[i].id = i as u16;
            r.frames[i].heap = heap;
        }

        r
    }

    pub fn push_frame(&mut self, m: Rp<ClassMember>) -> Rp<JFrame> {
        let empty = self.is_empty();
        let cur = if empty { Rp::null() } else { self.cur_frame() };

        let f = &mut self.frames[self.size];

        // TODO: use realloc to allocate memory
        // assign stack base
        // new stack base = prev stack base + prev max stack
        let new_base: Rp<u64> = if empty {
            self.stack_data.as_ptr().into()
        } else {
            unsafe { cur.stack_base.raw().add(cur.max_stack() as usize).into() }
        };

        f.reset(new_base, m);
        self.size += 1;
        f.into()
    }

    #[inline]
    pub fn pop_frame(&mut self) {
        self.size -= 1;
    }

    #[inline]
    pub fn back_frame(&self, i: usize) -> Rp<JFrame> {
        unsafe { self.frames.as_ptr().add(self.size - i).into() }
    }

    #[inline]
    pub fn prev_frame(&self) -> Rp<JFrame> {
        unsafe { self.frames.as_ptr().add(self.size - 2).into() }
    }

    #[inline]
    pub fn cur_frame(&self) -> Rp<JFrame> {
        unsafe { self.frames.as_ptr().add(self.size - 1).into() }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

#[derive(Default, Clone, Copy)]
pub struct JFrame {
    local_base: Rp<u64>,
    pub method: Rp<ClassMember>,
    pub heap: Rp<Heap>,
    pub next_pc: u32,
    id: u16,
    pub no_ret: bool,
    stack_base: Rp<u64>,
    stack_size: u16,
}

macro_rules! xx_ref {
    ($f: ident) => {
        pub fn $f(&mut self, i: usize) -> Rp<SymRef> {
            let mut cur = self.class().get_mut();
            let sym = { self.heap.$f(&mut cur, i) };
            sym
        }
    };
}

impl JFrame {
    #[inline]
    pub fn id(&self) -> u16 {
        self.id
    }

    #[inline]
    pub fn drop(&mut self, n: usize) {
        self.stack_size -= n as u16;
    }

    pub fn reset(&mut self, local_base: Rp<u64>, method: Rp<ClassMember>) {
        self.method = method;
        self.next_pc = 0;
        self.local_base = local_base;
        self.stack_base = unsafe { self.local_base.raw().add(self.max_locals() as usize).into() };
        self.stack_size = 0;
    }
    #[inline]
    pub fn pop_slots(&mut self, i: usize) -> &'static [u64] {
        let r = &self.slots()[self.stack_size as usize - i..self.stack_size as usize];
        self.stack_size -= i as u16;
        r
    }

    #[inline]
    pub fn local_vars(&self) -> &'static mut [u64] {
        self.local_base.as_slice(self.max_locals() as usize)
    }

    #[inline]
    pub fn slots(&self) -> &'static mut [u64] {
        self.stack_base.as_slice(self.max_stack() as usize)
    }

    #[inline]
    pub fn stack_size(&self) -> u16 {
        self.stack_size
    }

    #[inline]
    pub fn max_locals(&self) -> u16 {
        self.method.max_locals
    }

    #[inline]
    pub fn max_stack(&self) -> u16 {
        self.method.max_stack
    }

    #[inline]
    pub fn class(&self) -> Rp<Class> {
        self.method.class
    }

    #[inline]
    pub fn cp(&self) -> Rp<ConstantPool> {
        (&self.class().cp).into()
    }

    #[inline]
    pub fn this(&self) -> Rp<Object> {
        (self.local_vars()[0] as usize).into()
    }

    xx_ref!(class_ref);
    xx_ref!(field_ref);
    xx_ref!(method_ref);
    xx_ref!(iface_ref);

    pub fn pass_args(&mut self, other: &mut JFrame, arg_cells: usize) {
        let stack_data =
            &self.slots()[self.stack_size as usize - arg_cells..self.stack_size as usize];
        other.local_vars()[..arg_cells].copy_from_slice(stack_data);
        self.stack_size -= arg_cells as u16;
    }
}

macro_rules! pop_x {
    ($f: ident, $t: ty) => {
        #[inline]
        pub fn $f(&mut self) -> $t {
            self.pop_u32() as $t
        }
    };
}

macro_rules! pop_xx {
    ($f: ident, $t: ty) => {
        #[inline]
        pub fn $f(&mut self) -> $t {
            self.pop_u64() as $t
        }
    };
}

macro_rules! push_x {
    ($f: ident, $t: ty) => {
        #[inline]
        pub fn $f(&mut self, v: $t) {
            self.push_u32(v as u32);
        }
    };
}

macro_rules! push_xx {
    ($f: ident, $t: ty) => {
        #[inline]
        pub fn $f(&mut self, v: $t) {
            self.push_u64(v as u64)
        }
    };
}

impl JFrame {
    #[inline]
    pub fn push_u32(&mut self, v: u32) {
        self.stack_base[self.stack_size as usize] = v as u64;
        self.stack_size += 1;
    }

    push_x!(push_u8, u8);
    push_x!(push_u16, u16);
    push_x!(push_i32, i32);

    #[inline]
    pub fn push_u64(&mut self, v: u64) {
        self.stack_base[self.stack_size as usize] = v & 0xffffffff;
        self.stack_base[self.stack_size as usize + 1] = v >> 32;
        self.stack_size += 2;
    }

    push_xx!(push_i64, i64);

    #[inline]
    pub fn push_f32(&mut self, v: f32) {
        self.push_u32(v.to_bits());
    }

    #[inline]
    pub fn push_f64(&mut self, v: f64) {
        self.push_u64(v.to_bits());
    }

    #[inline]
    pub fn pop_u32(&mut self) -> u32 {
        let r = self.slots().get_u32(self.stack_size as usize - 1);
        self.stack_size -= 1;
        r
    }

    pop_x!(pop_u8, u8);
    pop_x!(pop_u16, u16);
    pop_x!(pop_i32, i32);

    #[inline]
    pub fn pop_u64(&mut self) -> u64 {
        let low = self.stack_base[self.stack_size as usize - 2];
        let high = self.stack_base[self.stack_size as usize - 1];
        self.stack_size -= 2;
        high << 32 | low
    }

    pop_xx!(pop_i64, i64);

    #[inline]
    pub fn pop_f32(&mut self) -> f32 {
        f32::from_bits(self.pop_u32())
    }

    #[inline]
    pub fn pop_f64(&mut self) -> f64 {
        f64::from_bits(self.pop_u64())
    }

    #[inline]
    pub fn push_null(&mut self) {
        self.slots()[self.stack_size as usize] = 0;
        self.stack_size += 1;
    }

    #[inline]
    pub fn push_slot(&mut self, v: u64) {
        self.stack_base[self.stack_size as usize] = v;
        self.stack_size += 1;
    }

    #[inline]
    pub fn pop_slot(&mut self) -> u64 {
        let r = self.slots()[self.stack_size as usize - 1];
        self.stack_size -= 1;
        r
    }

    #[inline]
    pub fn pop_obj(&mut self) -> Rp<Object> {
        (self.pop_slot() as usize).into()
    }

    #[inline]
    pub fn push_obj(&mut self, obj: Rp<Object>) {
        self.push_slot(obj.ptr() as u64)
    }

    #[inline]
    pub fn back_obj(&self, i: usize) -> Rp<Object> {
        (self.stack_base[self.stack_size as usize - i] as usize).into()
    }
}

pub(crate) trait DupStack {
    fn dup(&mut self);
    fn dup2(&mut self);
    fn dup_x1(&mut self);
    fn dup_x2(&mut self);
    fn dup2_x1(&mut self);
    fn dup2_x2(&mut self);
    fn swap(&mut self);
}

impl DupStack for JFrame {
    fn dup(&mut self) {
        let top = { self.stack_base[self.stack_size as usize - 1] };
        self.push_slot(top);
    }

    fn dup2(&mut self) {
        let (v2, v1) = {
            (
                self.stack_base[self.stack_size as usize - 2],
                self.stack_base[self.stack_size as usize - 1],
            )
        };
        self.stack_base[self.stack_size as usize] = v2;
        self.stack_base[self.stack_size as usize + 1] = v1;
        self.stack_size += 2;
    }

    fn dup_x1(&mut self) {
        let v1 = self.stack_base[self.stack_size as usize - 1];
        let v2 = self.stack_base[self.stack_size as usize - 2];
        self.stack_base[self.stack_size as usize - 1] = v2;
        self.stack_base[self.stack_size as usize - 2] = v1;
        self.push_slot(v1);
    }

    fn dup_x2(&mut self) {
        let v1 = self.stack_base[self.stack_size as usize - 1];
        let v2 = self.stack_base[self.stack_size as usize - 2];
        let v3 = self.stack_base[self.stack_size as usize - 3];
        self.stack_base[self.stack_size as usize - 1] = v2;
        self.stack_base[self.stack_size as usize - 2] = v3;
        self.stack_base[self.stack_size as usize - 3] = v1;
        self.push_slot(v1);
    }

    fn dup2_x1(&mut self) {
        let v1 = self.stack_base[self.stack_size as usize - 1];
        let v2 = self.stack_base[self.stack_size as usize - 2];
        let v3 = self.stack_base[self.stack_size as usize - 3];
        self.stack_base[self.stack_size as usize - 1] = v3;
        self.stack_base[self.stack_size as usize - 2] = v1;
        self.stack_base[self.stack_size as usize - 3] = v2;
        self.stack_base[self.stack_size as usize] = v2;
        self.stack_base[self.stack_size as usize + 1] = v1;
        self.stack_size += 2;
    }

    fn dup2_x2(&mut self) {
        let v1 = self.stack_base[self.stack_size as usize - 1];
        let v2 = self.stack_base[self.stack_size as usize - 2];
        let v3 = self.stack_base[self.stack_size as usize - 3];
        let v4 = self.stack_base[self.stack_size as usize - 4];
        self.stack_base[self.stack_size as usize - 1] = v3;
        self.stack_base[self.stack_size as usize - 2] = v4;
        self.stack_base[self.stack_size as usize - 3] = v1;
        self.stack_base[self.stack_size as usize - 4] = v2;
        self.stack_base[self.stack_size as usize] = v2;
        self.stack_base[self.stack_size as usize + 1] = v1;
        self.stack_size += 2;
    }

    fn swap(&mut self) {
        let v1 = self.stack_base[self.stack_size as usize - 1];
        let v2 = self.stack_base[self.stack_size as usize - 2];
        self.stack_base[self.stack_size as usize - 1] = v2;
        self.stack_base[self.stack_size as usize - 2] = v1;
    }
}
