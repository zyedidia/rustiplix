use core::slice;
use kernel::board;
use kernel::crc::crc32;
use kernel::dev::uart::Uart;
use kernel::timer;

use crate::bootloader::BootData;

#[repr(u32)]
enum BootFlags {
    BootStart = 0xFFFF0000,

    GetProgInfo = 0x11223344,
    PutProgInfo = 0x33334444,

    GetCode = 0x55556666,
    PutCode = 0x77778888,

    BootSuccess = 0x9999AAAA,
    BootError = 0xBBBBCCCC,

    BadCodeAddr = 0xdeadbeef,
    BadCodeCksum = 0xfeedface,
}

fn rx_u64<U: Uart>(uart: &mut U) -> u64 {
    (uart.rx() as u64)
        | ((uart.rx() as u64) << 8)
        | ((uart.rx() as u64) << 16)
        | ((uart.rx() as u64) << 24)
        | ((uart.rx() as u64) << 32)
        | ((uart.rx() as u64) << 40)
        | ((uart.rx() as u64) << 48)
        | ((uart.rx() as u64) << 56)
}

fn rx_u32<U: Uart>(uart: &mut U) -> u32 {
    (uart.rx() as u32)
        | ((uart.rx() as u32) << 8)
        | ((uart.rx() as u32) << 16)
        | ((uart.rx() as u32) << 24)
}

fn tx_u32<U: Uart>(u: u32, uart: &mut U) {
    uart.tx(u as u8);
    uart.tx((u >> 8) as u8);
    uart.tx((u >> 16) as u8);
    uart.tx((u >> 24) as u8);
}

use core::ops::DerefMut;

pub fn recv() -> BootData {
    extern "C" {
        static mut _heap_start: u8;
    }

    let heap = unsafe { &mut _heap_start as *mut u8 };

    let mut guard = board::UART.lock();
    let uart = guard.deref_mut().deref_mut();
    loop {
        tx_u32(BootFlags::GetProgInfo as u32, uart);
        timer::delay_us(100 * 1000);
        if !uart.rx_empty() && rx_u32(uart) == BootFlags::PutProgInfo as u32 {
            break;
        }
    }

    let entry = rx_u64(uart);
    let nbytes = rx_u32(uart) as usize;
    let crc_recv = rx_u32(uart);

    tx_u32(BootFlags::GetCode as u32, uart);
    tx_u32(crc_recv, uart);

    if rx_u32(uart) != BootFlags::PutCode as u32 {
        tx_u32(BootFlags::BootError as u32, uart);
        panic!("Boot error: didn't receive PutCode after GetCode");
    }

    for i in 0..nbytes {
        unsafe { heap.add(i).write_volatile(uart.rx()) };
    }

    let data = unsafe { slice::from_raw_parts(heap, nbytes) };

    let crc_calc = crc32(data);
    if crc_calc != crc_recv {
        tx_u32(BootFlags::BadCodeCksum as u32, uart);
        panic!("Boot error: checksum mismatch");
    }

    tx_u32(BootFlags::BootSuccess as u32, uart);
    uart.tx_flush();

    BootData {
        entry: entry as *mut u8,
        data,
    }
}
