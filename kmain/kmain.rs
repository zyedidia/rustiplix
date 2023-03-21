use kernel::println;

pub fn kmain(_coreid: u32) {
    use kernel::arch::riscv64::monitor::init::enter_smode;
    enter_smode();

    println!("hi");
    println!("hello world {}", 42);
}
