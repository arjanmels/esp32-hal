#![no_std]
#![no_main]

extern crate esp32_hal as hal;
extern crate panic_halt;
extern crate xtensa_lx6_rt;

use hal::prelude::*;
use xtensa_lx6_rt::get_cycle_count;

/// The default clock source is the onboard crystal
/// In most cases 40mhz (but can be as low as 2mhz depending on the board)
const CORE_HZ: u32 = 40_000_000;

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { hal::esp32::Peripherals::steal() };

    let mut rtccntl = dp.RTCCNTL;
    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the wdt's on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);
    disable_rtc_wdt(&mut rtccntl);

    let pins = dp.GPIO.split();
    let mut led = pins.gpio2.into_push_pull_output(); // An open drain or push pull may be used in this instance (next line).
    // let mut led = pins.gpio2.into_open_drain_output();

    loop {
        led.set_high().unwrap();
        delay(CORE_HZ);
        led.set_low().unwrap();
        delay(CORE_HZ);
    }
}

fn disable_rtc_wdt(rtccntl: &mut hal::esp32::RTCCNTL) {
    /* Disables the RTCWDT */
    rtccntl
        .wdtwprotect
        .write(|w| unsafe { w.bits(WDT_WKEY_VALUE) });
    rtccntl.wdtconfig0.modify(|_, w| unsafe {
        w.wdt_stg0()
            .bits(0x0)
            .wdt_stg1()
            .bits(0x0)
            .wdt_stg2()
            .bits(0x0)
            .wdt_stg3()
            .bits(0x0)
            .wdt_flashboot_mod_en()
            .clear_bit()
            .wdt_en()
            .clear_bit()
    });
    rtccntl.wdtwprotect.write(|w| unsafe { w.bits(0x0) });
}

fn disable_timg_wdts(timg0: &mut hal::esp32::TIMG0, timg1: &mut hal::esp32::TIMG1) {
    timg0
        .wdtwprotect
        .write(|w| unsafe { w.bits(WDT_WKEY_VALUE) });
    timg1
        .wdtwprotect
        .write(|w| unsafe { w.bits(WDT_WKEY_VALUE) });

    timg0.wdtconfig0.write(|w| unsafe { w.bits(0x0) });
    timg1.wdtconfig0.write(|w| unsafe { w.bits(0x0) });
}

/// cycle accurate delay using the cycle counter register
pub fn delay(clocks: u32) {
    // NOTE: does not account for rollover
    let target = get_cycle_count() + clocks;
    loop {
        if get_cycle_count() > target {
            break;
        }
    }
}
