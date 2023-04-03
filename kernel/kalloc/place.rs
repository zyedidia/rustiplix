#[macro_export]
macro_rules! pbox {
    ($x: expr) => {{
        let v = ::alloc::vec![$x];
        let b = v.into_boxed_slice();
        let ptr = (::alloc::boxed::Box::into_raw(b)).cast();
        // SAFETY: b was a Box<[T]> of length 1, which is identical in layout to Box<T>
        unsafe { ::alloc::boxed::Box::from_raw(ptr) }
    }};
}
