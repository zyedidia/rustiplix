pub mod brkpt {
    pub const LOAD: usize = 1 << 0;
    pub const STORE: usize = 1 << 1;
    pub const EXEC: usize = 1 << 2;
    pub const USER: usize = 1 << 3;
    pub const SUPER: usize = 1 << 4;
    pub const EQ: usize = 0x0 << 7;
    pub const GE: usize = 0x2 << 7;
    pub const LT: usize = 0x3 << 7;
    pub const CHAIN: usize = 1 << 11;
    pub const MATCH6: usize = 6 << 60;
}

pub const NBRKPT: usize = 2;

pub fn place(n: usize, addr: usize, flags: usize) {
    csr!(tselect = n);
    csr!(tdata1 = flags | brkpt::MATCH6);
    csr!(tdata2 = addr);
}

fn clear(n: usize) {
    csr!(tselect = n);
    csr!(tdata1 = 0 | brkpt::MATCH6);
}

fn place_mismatch(n: usize, addr: usize, flags: usize) {
    // < addr
    csr!(tselect = n);
    csr!(tdata1 = flags | brkpt::LT | brkpt::MATCH6);
    csr!(tdata2 = addr);

    // >= addr + 2
    csr!(tselect = n + 1);
    csr!(tdata1 = flags | brkpt::GE | brkpt::MATCH6);
    csr!(tdata2 = addr + 2);
}

fn clear_all() {
    for i in 0..NBRKPT {
        clear(i);
    }
}
