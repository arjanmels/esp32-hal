#![no_std]
#![no_main]
#![feature(asm)]

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

    esp32_hal::watchdog_disabler::disable();
    Console::begin(19200);

    let gpios = dp.GPIO.split();
    let mut blinky1 = gpios.gpio13.into_push_pull_output();
    let mut blinky2 = gpios.gpio2.into_push_pull_output();

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

/// Basic panic handler - just loops
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
