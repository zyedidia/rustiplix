#[no_mangle]
pub unsafe extern "C" fn memset(v: *mut u8, c: u8, n: usize) -> *mut u8 {
    let mut p = v;
    for _ in 0..n {
        *p = c;
        p = p.add(1);
    }
    return p;
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, mut n: usize) -> *mut u8 {
    let mut s = src;
    let mut d = dst;
    while n > 0 {
        *d = *s;

        n -= 1;
        s = s.add(1);
        d = d.add(1);
    }
    return dst;
}
