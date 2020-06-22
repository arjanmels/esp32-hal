#![no_std]
#![no_main]

extern crate esp32_hal as hal;
extern crate xtensa_lx6_rt;

use {
    core::{fmt::Write, panic::PanicInfo},
    hal::{
        clock_control::{self, sleep, CPUSource, ClockControl, ClockControlConfig},
        dport::Split,
        dprintln, i2c,
        prelude::*,
        serial::{config::Config as SerialConfig, Serial},
        timer::Timer,
    },
    ssd1306::{prelude::*, Builder},
};

#[no_mangle]
fn main() -> ! {
    let dp = esp32::Peripherals::take().unwrap();

    let (mut dport, dport_clock_control) = dp.DPORT.split();

    // setup clocks & watchdog
    let mut clkcntrl = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        clock_control::XTAL_FREQUENCY_AUTO,
    )
    .unwrap();

    // set desired clock frequencies
    clkcntrl
        .set_cpu_frequencies(
            CPUSource::PLL,
            80.MHz(),
            CPUSource::PLL,
            240.MHz(),
            CPUSource::PLL,
            80.MHz(),
        )
        .unwrap();

    let (clkcntrl_config, mut watchdog) = clkcntrl.freeze().unwrap();
    watchdog.disable();
    let (_, _, _, mut watchdog0) = Timer::new(dp.TIMG0, clkcntrl_config);
    watchdog0.disable();
    let (_, _, _, mut watchdog1) = Timer::new(dp.TIMG1, clkcntrl_config);
    watchdog1.disable();

    let pins = dp.GPIO.split();

    let mut serial: Serial<_, _, _> = Serial::new(
        dp.UART0,
        esp32_hal::serial::Pins {
            tx: pins.gpio1,
            rx: pins.gpio3,
            cts: None,
            rts: None,
        },
        SerialConfig::default().baudrate(115200.into()),
        clkcntrl_config,
        &mut dport,
    )
    .unwrap();

    writeln!(serial, "\n\n\nserial initialized").unwrap();

    let mut rst = pins.gpio16.into_push_pull_output();

    let mut disp: GraphicsMode<_> = {
        let i2c = i2c::I2C::new(
            dp.I2C0,
            i2c::Pins {
                sda: pins.gpio4,
                scl: pins.gpio15,
            },
            &mut dport,
        );
        Builder::new().connect_i2c(i2c).into()
    };
    writeln!(serial, "display built").unwrap();
    rst.set_low().unwrap();
    sleep(1.s());
    rst.set_high().unwrap();

    disp.init().unwrap();
    writeln!(serial, "display initialised").unwrap();
    disp.set_pixel(10, 10, 1);
    disp.set_pixel(20, 20, 1);
    disp.flush().unwrap();
    writeln!(serial, "display flushed").unwrap();

    loop {
        writeln!(serial, "tick").unwrap();
        sleep(500.ms());
        writeln!(serial, "tock").unwrap();
        sleep(500.ms());
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // park the other core
    unsafe { ClockControlConfig {}.park_core(esp32_hal::get_other_core()) };

    // print panic message
    dprintln!("\n\n*** {:?}", info);

    // park this core
    unsafe { ClockControlConfig {}.park_core(esp32_hal::get_core()) };

    dprintln!("Not reached because core is parked.");

    // this statement will not be reached, but is needed to make this a diverging function
    loop {}
}
