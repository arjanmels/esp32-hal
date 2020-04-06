use crate::prelude::*;
use crate::dport::Split;

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

/// At some future time we will have to do something with the watchdog timer, but for now, this is an easy way to disable it.
pub fn disable() {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    let (_dport, dport_clock_control) = dp.DPORT.split();

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timer on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);

    let clkcntrl = crate::clock_control::ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        crate::clock_control::XTAL_FREQUENCY_AUTO,
    )
        .unwrap();

    let (_clkcntrl_config, mut watchdog) = clkcntrl.freeze().unwrap();
    watchdog.disable();

}

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
