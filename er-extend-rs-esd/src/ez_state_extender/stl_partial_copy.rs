#[repr(C)]
#[derive(PartialEq, Debug)]
/// Represents a C++ span, a collection that doesn't own its data.
pub struct DynamicSizeSpan<T> {
    data: *const T,
    length: usize,
}

impl<T> Default for DynamicSizeSpan<T> {
    fn default() -> Self {
        Self {
            data: 0x0 as _,
            length: 0,
        }
    }
}

impl<T> DynamicSizeSpan<T> {
    pub fn as_ptr(&self) -> *const T {
        self.data
    }

    pub fn as_slice(&self) -> &[T] {
        if let Some(ptr) = unsafe { self.data.as_ref() } {
            unsafe { std::slice::from_raw_parts(ptr as _, self.length) }
        } else {
            &[]
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_slice().iter()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// # Safety
    ///
    /// Caller must ensure the memory is valid and of size `self.size`
    pub unsafe fn as_mut(&mut self) -> &mut [T] {
        if let Some(ptr) = unsafe{ self.data.as_ref() } {
            unsafe { std::slice::from_raw_parts_mut(ptr as *const _ as *mut _, self.length) }
        } else {
            &mut []
        }
    }

    /// # Safety
    ///
    /// Caller must ensure the memory is valid and of size `self.size` and that access is
    /// exclusive.
    pub unsafe fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        unsafe { self.as_mut() }.iter_mut()
    }

    /// # Safety
    ///
    /// Caller must ensure that the data argument is populated with a valid non-null pointer, that
    /// type T is complete and properly sized and the size param represents the proper count for
    /// the collection.
    pub unsafe fn from_raw_parts(data: *const T, length: usize) -> Self {
        Self { data, length }
    }

    pub const fn from_static_slice(v: &'static [T]) -> Self {
        Self {
            data: v.as_ptr(),
            length: v.len(),
        }
    }

    pub const fn empty() -> Self {
        Self { data: 0x0 as _, length: 0 }
    }
}
