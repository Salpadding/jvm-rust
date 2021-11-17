use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

// the memory safety is guraranteed by programmer, not compiler
pub trait Unmanged: Sized {
    #[inline]
    fn as_rp(&self) -> Rp<Self> {
        Rp::from_ref(&self)
    }
}

pub type Np<T> = Rp<T>;

// unsafe raw pointer wrapper, which is also thread unsafe
// for escape compiler check
pub struct Rp<T: Unmanged> {
    p: PhantomData<T>,
    ptr: usize,
}

impl<T: Debug + Unmanged> Debug for Rp<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            f.write_str("NULL")
        } else {
            write!(f, "{:?}", self.as_ref())
        }
    }
}

impl<T: Unmanged> Clone for Rp<T> {
    fn clone(&self) -> Self {
        Self {
            p: PhantomData,
            ptr: self.ptr,
        }
    }
}

impl<T: Unmanged> Copy for Rp<T> {}

impl<T: Unmanged> Default for Rp<T> {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

impl<T: Unmanged> Deref for Rp<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &'static T {
        self.get_mut()
    }
}

impl<T: Unmanged> DerefMut for Rp<T> {
    #[inline]
    fn deref_mut(&mut self) -> &'static mut T {
        self.get_mut()
    }
}

impl<T: Unmanged> AsRef<T> for Rp<T> {
    #[inline]
    fn as_ref(&self) -> &'static T {
        self.get_mut()
    }
}

impl<T: Unmanged> Rp<T> {
    #[inline]
    pub fn from_ref(x: &T) -> Rp<T> {
        {
            Rp {
                p: PhantomData,
                ptr: x as *const T as usize,
            }
        }
    }

    #[inline]
    pub fn from_ptr(p: usize) -> Rp<T> {
        if p == 0 {
            return Rp::null();
        }
        Rp {
            p: PhantomData,
            ptr: p,
        }
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.ptr == 0
    }

    #[inline]
    pub fn null() -> Rp<T> {
        Rp {
            p: PhantomData,
            ptr: 0usize,
        }
    }

    // allocate on heap
    #[inline]
    pub fn new(x: T) -> Self {
        let b = Box::new(x);
        let l = Box::leak(b);
        Self {
            ptr: l as *mut T as usize,
            p: PhantomData,
        }
    }

    #[inline]
    pub fn get_mut(&self) -> &'static mut T {
        if self.is_null() {
            panic!("null pointer");
        }
        unsafe { &mut *(self.ptr as *mut T) }
    }

    #[inline]
    pub fn raw(&self) -> *mut T {
        self.ptr as *mut T
    }

    #[inline]
    pub fn drop(&mut self) {
        if self.is_null() {
            return;
        }
        let b = unsafe { Box::from_raw(self.ptr as *mut T) };
        std::mem::drop(b);
        self.ptr = 0usize;
    }

    #[inline]
    pub fn ptr(&self) -> usize {
        self.ptr
    }
}
