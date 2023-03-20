#[no_mangle]
pub unsafe extern "C" fn memset(v: *mut u8, c: u8, n: usize) -> *mut u8 {
    let mut p = v;
    for _ in 0..n {
        *p = c;
        p = p.add(1);
    }
    return p;
}
