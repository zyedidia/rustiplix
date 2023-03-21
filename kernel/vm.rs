pub mod perm {
    pub const READ: u8 = 1 << 0;
    pub const WRITE: u8 = 1 << 1;
    pub const EXEC: u8 = 1 << 2;
    pub const USER: u8 = 1 << 3;
    pub const COW: u8 = 1 << 4;
}
