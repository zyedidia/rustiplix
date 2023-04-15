use crate::arch::timer;

/// Spin for 'amt' where the 'time' function returns the current time.
fn delay(amt: u64, time: fn() -> u64) {
    let rb = time();
    loop {
        let ra = time();
        if (ra - rb) >= amt {
            break;
        }
    }
}

/// Spin for 'cyc' cycles.
pub fn delay_cycles(cyc: u64) {
    delay(cyc, timer::cycles);
}

/// Spin for 'us' microseconds.
pub fn delay_us(us: u64) {
    delay(us * timer::freq() / 1_000_000, timer::time);
}

/// Returns the current time in a platform specific frequency.
pub fn time() -> u64 {
    timer::time()
}

/// Calculates the microseconds since a previous time. Converts from "times" to microseconds by
/// using timer::freq.
pub fn us_since(prev: u64) -> u64 {
    (timer::time() - prev) * 1_000_000 / timer::freq()
}
