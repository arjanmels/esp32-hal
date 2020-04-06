#![no_std]
#![no_main]
#![feature(asm)]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::clock_control::sleep;

use esp32_hal::println;
use esp32_hal::print;
use esp32_hal::console::Console;


const BLINK_HZ: Hertz = Hertz(2);

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timer on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);

    let gpios = dp.GPIO.split();
    let mut blinky1 = gpios.gpio13.into_push_pull_output();
    let mut blinky2 = gpios.gpio2.into_push_pull_output();

    Console::begin(19200);

    println!("\n\nESP32 Started\n\n");

    loop {
        print!("Blink high.");
        blinky1.set_high().unwrap();
        blinky2.set_high().unwrap();
        sleep((Hertz(1_000_000) / BLINK_HZ).us());
        println!("..low");
        blinky1.set_low().unwrap();
        blinky2.set_low().unwrap();
        sleep((Hertz(1_000_000) / BLINK_HZ).us());
    }
}

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

fn disable_timg_wdts(timg0: &mut esp32::TIMG0, timg1: &mut esp32::TIMG1) {
    timg0
        .wdtwprotect
        .write(|w| unsafe { w.bits(WDT_WKEY_VALUE) });
    timg1
        .wdtwprotect
        .write(|w| unsafe { w.bits(WDT_WKEY_VALUE) });

    timg0.wdtconfig0.write(|w| unsafe { w.bits(0x0) });
    timg1.wdtconfig0.write(|w| unsafe { w.bits(0x0) });
}

/// Basic panic handler - just loops
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
