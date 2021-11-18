use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

// the memory safety is guraranteed by programmer, not compiler
pub trait Unmanaged: Sized {
    #[inline]
    fn as_rp(&self) -> Rp<Self> {
        Rp::from_ref(&self)
    }
}

macro_rules! im {
    ($x: ident) => {
        impl Unmanaged for $x {}
    };
}

// primitives is always unmanaged
im!(u64);
im!(i64);
im!(i32);
im!(u32);
im!(bool);
im!(f32);
im!(f64);
im!(char);
im!(u8);
im!(i8);
im!(u16);
im!(i16);

// unsafe raw pointer wrapper, which is also thread unsafe
// for escape compiler check
pub struct Rp<T: Unmanaged> {
    p: PhantomData<T>,
    ptr: usize,
}

impl<T: Debug + Unmanaged> Debug for Rp<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            f.write_str("NULL")
        } else {
            write!(f, "{:?}", self.as_ref())
        }
    }
}

impl<T: Unmanaged> Clone for Rp<T> {
    fn clone(&self) -> Self {
        Self {
            p: PhantomData,
            ptr: self.ptr,
        }
    }
}

impl<T: Unmanaged> Copy for Rp<T> {}

impl<T: Unmanaged> Default for Rp<T> {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

impl<T: Unmanaged> Deref for Rp<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &'static T {
        self.get_mut()
    }
}

impl<T: Unmanaged> DerefMut for Rp<T> {
    #[inline]
    fn deref_mut(&mut self) -> &'static mut T {
        self.get_mut()
    }
}

impl<T: Unmanaged> AsRef<T> for Rp<T> {
    #[inline]
    fn as_ref(&self) -> &'static T {
        self.get_mut()
    }
}

impl<T: Unmanaged> std::ops::Index<usize> for Rp<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*(self.ptr as *mut T).add(index) }
    }
}

impl<T: Unmanaged> std::ops::IndexMut<usize> for Rp<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *(self.ptr as *mut T).add(index) }
    }
}

impl<T: Unmanaged> Rp<T> {
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

    // drop is only available for Rp created by new
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

impl<T: Unmanaged + Copy + Default> Rp<T> {
    pub fn new_vec(size: usize) -> Self {
        let b: Vec<T> = vec![T::default(); size];
        let p = (*b).as_ptr() as usize;
        std::mem::forget(b);
        Self {
            ptr: p,
            p: PhantomData,
        }
    }

    pub fn drop_vec(&mut self, size: usize) {
        let b = unsafe { Vec::from_raw_parts(self.ptr as *mut T, size, size) };
        self.ptr = 0;
        std::mem::drop(b);
    }
}
