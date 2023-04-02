// Simple implementation of crc32: https://git.shipyard.rs/jstrong/const-crc32.

#[rustfmt::skip]
const fn table_fn(i: u32) -> u32 {
    let mut out = i;
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out = if out & 1 == 1 { 0xedb88320 ^ (out >> 1) } else { out >> 1 };
    out
}
const fn get_table() -> [u32; 256] {
    let mut table: [u32; 256] = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = table_fn(i as u32);
        i += 1;
    }
    table
}
const TABLE: [u32; 256] = get_table();

pub const fn crc32(buf: &[u8]) -> u32 {
    let mut out = !0u32;
    let mut i = 0usize;
    while i < buf.len() {
        out = (out >> 8) ^ TABLE[((out & 0xff) ^ (buf[i] as u32)) as usize];
        i += 1;
    }
    !out
}
