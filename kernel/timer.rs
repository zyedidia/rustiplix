use crate::arch::timer;

fn delay(amt: u64, time: fn() -> u64) {
    let rb = time();
    loop {
        let ra = time();
        if (ra - rb) >= amt {
            break;
        }
    }
}

pub fn delay_cycles(cyc: u64) {
    delay(cyc, timer::cycles);
}

pub fn delay_us(us: u64) {
    delay(us * timer::freq() / 1_000_000, timer::time);
}
