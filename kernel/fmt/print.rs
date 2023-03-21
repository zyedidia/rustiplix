#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
            use core::fmt::Write;
            let mut uart = kernel::board::virt::UART.lock();
            let _ = write!(uart, $($args)+);
    });
}

#[macro_export]
macro_rules! println
{
    () => ({
        use kernel::print;
        print!("\r\n")
    });
    ($fmt:expr) => ({
        use kernel::print;
        print!(concat!($fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        use kernel::print;
        print!(concat!($fmt, "\r\n"), $($args)+)
    });
}
