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
