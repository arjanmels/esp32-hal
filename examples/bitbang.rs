#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
use esp32_hal::prelude::*;
use esp32_hal::println;
use esp32_hal::console::Console;
use embedded_hal::digital::v2::{OutputPin,InputPin};
use xtensa_lx6_rt::get_cycle_count;
use nb::Result;
use esp32_hal::gpio::{InputOutput, PushPull, Output, OpenDrain};
// use esp32_hal::gpio_xx::{Gpio2, Output, PushPull, Gpio15};
// use esp32_hal::gpio_xx::{Gpio2, Output, PushPull, Gpio3};

/// The default clock source is the onboard crystal
/// In most cases 40mhz (but can be as low as 2mhz depending on the board)
const CORE_HZ: u32 = 40_000_000;

/// This example shows how to use a gpio_xx pin in 2 directions. Set it and check it.
/// It also shows how to store the pin in a struct for use in other places.
/// To run this example connect a button (from +5v) and a led (to gnd) to gpio_xx 15.
/// While you push the button the led will be on. After you push the button, it will give 2
/// flashes to acknowledge the push.

pub struct ButtonFlasher<SysLedGpio, BiDirGpio>
where
    SysLedGpio: OutputPin,
    BiDirGpio: OutputPin + InputPin,
{
    pub sys_led: SysLedGpio,
    pub button_with_led: BiDirGpio,
    pub last_val:bool
}

impl <SysLedGpio, BiDirGpio, E> ButtonFlasher <SysLedGpio, BiDirGpio>
where
    SysLedGpio: OutputPin<Error = E>,
    BiDirGpio: OutputPin<Error = E> + InputPin<Error = E>
{
    fn new(sys_led:SysLedGpio, button_with_led:BiDirGpio) -> ButtonFlasher <SysLedGpio, BiDirGpio>{
        let last_val = match button_with_led.is_high() { Ok(v) => v, Err(_) => false};

        ButtonFlasher {
            sys_led,
            button_with_led,
            last_val: last_val
        }
    }

    fn watch(&mut self)  -> Result<(), <BiDirGpio as embedded_hal::digital::v2::OutputPin>::Error> {
        let val = self.button_with_led.is_high()?;
        if self.last_val != val {
            if val {
                println!("Button pushed.");
                self.sys_led.set_high()?;
            } else {
                println!("Button released.");
                self.sys_led.set_low()?;

                // Flash the led 2 times.
                delay(CORE_HZ);
                self.button_with_led.set_high()?;println!("Blink");
                delay(CORE_HZ);
                self.button_with_led.set_low()?;
                delay(CORE_HZ);
                self.button_with_led.set_high()?;println!("Blink");
                delay(CORE_HZ);
                self.button_with_led.set_low()?;
                println!("Monitoring again");
            }
            self.last_val = val;
        }

        Ok(())
    }
}

#[no_mangle]
#[allow(unused_variables, unused_mut, unused_macros)]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };
    esp32_hal::watchdog_disabler::disable();
    Console::begin(19200);

    let gpios = dp.GPIO.split();
    let mut sys_led:Output<PushPull> = gpios.gpio2.into_push_pull_output();
    let mut button_with_led:InputOutput<OpenDrain> = gpios.gpio15.into_open_drain_output();

    // let mut v = gpios.gpio18.into_pull_down();
    // let mut w = gpios.gpio20.into_push_pull_start_low();
    // let mut x = gpios.gpio21.into_pull_up();
    // let mut y = gpios.gpio22.into_floating();
    // let mut z = gpios.gpio23.into_open_drain();

    let mut flasher = ButtonFlasher::new(sys_led, button_with_led);
    println!("Monitoring button. Press it and the led connected to the same pin will blink twice.");
    loop {
        flasher.watch().unwrap();

    }
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
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n\n*** {:?}", info);
    loop {}
}
