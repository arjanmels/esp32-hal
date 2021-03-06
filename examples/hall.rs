#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use esp32_hal::prelude::*;

use esp32_hal::analog::adc::ADC;
use esp32_hal::analog::config::{Adc1Config, Attenuation};
use esp32_hal::clock_control::sleep;
use esp32_hal::dport::Split;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timer on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);

    let clkcntrl = esp32_hal::clock_control::ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        esp32_hal::clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    let (clkcntrl_config, mut watchdog) = clkcntrl.freeze().unwrap();
    watchdog.disable();

    /* Setup serial connection */
    let serial = Serial::uart0(
        dp.UART0,
        (NoTx, NoRx),
        Config::default(),
        clkcntrl_config,
        &mut dport,
    )
    .unwrap();
    let (mut tx, _rx) = serial.split();

    /* Set ADC pins to analog mode */
    let gpios = dp.GPIO.split();
    let mut pin36 = gpios.gpio36.into_analog();
    let mut pin39 = gpios.gpio39.into_analog();

    /* In the configuration enable hall sensor and its pins (36 and 39) */
    let mut adc_config = Adc1Config::new();
    adc_config.enable_hall_sensor();
    adc_config.enable_pin(&pin36, Attenuation::Attenuation0dB);
    adc_config.enable_pin(&pin39, Attenuation::Attenuation0dB);

    /* Hall sensor is only available on the ADC1 */
    let analog = dp.SENS.split();
    let mut adc1 = ADC::adc1(analog.adc1, adc_config).unwrap();

    loop {
        /* Read the sensor and print out the raw value once per second */
        let hall_sensor_value: i32 = adc1.read_hall_sensor(&mut pin36, &mut pin39);
        writeln!(tx, "Hall sensor raw value: {:?}", hall_sensor_value).unwrap();

        sleep(1.s());
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
