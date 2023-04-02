use crate::arch::timer;

pub fn delay_cycles(cyc: u64) {
    let rb = timer::cycles();
    loop {
        let ra = timer::cycles();
        if (ra - rb) >= cyc {
            break;
        }
    }
}

pub fn delay_us(us: u64) {
    let t = us * timer::freq() / 1_000_000;
    let rb = timer::time();
    loop {
        let ra = timer::time();
        if (ra - rb) >= t {
            break;
        }
    }
}
