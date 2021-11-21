use crate::{heap::class::ClassMember, rp::Rp};

pub struct JThread {
    pc: u32,
    next_pc: Option<i32>,

    // for all frame
    data: Vec<u64>,
}

pub struct JFrame {
    // frame id
    id: u16,
    stack_size: u16,
    method: Rp<ClassMember>,

    next_pc: u32,

    // data len = max_locals + max_stack
    data_off: usize,
}

impl JFrame {
    #[inline]
    fn ptr(&self) -> *mut u64 {
        self.data_off as *mut u64
    }

    #[inline]
    fn locals(&self) -> &mut [u64] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr(), self.max_locals()) }
    }

    #[inline]
    fn stack(&self, i: usize) -> u64 {
        unsafe { *self.ptr().add(self.max_locals() + i) }
    }

    #[inline]
    fn max_locals(&self) -> usize {
        self.method.max_locals
    }

    #[inline]
    fn max_stack(&self) -> usize {
        self.method.max_stack
    }
}
