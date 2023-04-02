#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
            use ::core::fmt::Write;
            let mut uart = $crate::board::UART.lock();
            let _ = write!(uart, $($args)+);
    });
}

#[macro_export]
macro_rules! println
{
    () => ({
        use $crate::print;
        print!("\n")
    });
    ($fmt:expr) => ({
        use $crate::print;
        print!(concat!($fmt, "\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        use $crate::print;
        print!(concat!($fmt, "\n"), $($args)+)
    });
}
