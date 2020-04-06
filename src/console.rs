//! Print debug information to UART0
//!
//! Directly writes to the UART0 TX uart queue.
//! This is unsafe! It is asynchronous with normal UART0 usage and
//! interrupts are not disabled.

use esp32::{UART0};
use crate::serial::{config::Config, *}; //NoRx, NoTx, Serial, Tx};
// use crate::serial::*;
use crate::dport::Split;
use crate::serial::config::{DataBits, Parity, StopBits};
use core::marker::PhantomData;
use embedded_hal::serial::Write;

pub struct Console {
    pub started:bool,
    pub tx:Tx<UART0>
}

pub enum Error {}

impl Console {
    pub fn begin(baud:u32) {
        let dp = unsafe { esp32::Peripherals::steal() };

        let (mut dport, dport_clock_control) = dp.DPORT.split();

        let clkcntrl = crate::clock_control::ClockControl::new(
            dp.RTCCNTL,
            dp.APB_CTRL,
            dport_clock_control,
            crate::clock_control::XTAL_FREQUENCY_AUTO,
        )
            .unwrap();

        let (clkcntrl_config, _watchdog) = clkcntrl.freeze().unwrap();

        let serial = Serial::uart0(
            dp.UART0,
            (NoTx, NoRx),
            Config {
                baudrate: crate::units::Hertz(baud),
                data_bits: DataBits::DataBits8,
                parity: Parity::ParityNone,
                stop_bits: StopBits::STOP1,
            }, // default configuration is 19200 baud, 8 data bits, 1 stop bit & no parity (8N1)
            clkcntrl_config,
            &mut dport,
        ).unwrap();

        let (tx, _rx) = serial.split();
        unsafe {
            CONSOLE.tx = tx;
            CONSOLE.started = true;
        }
    }
    pub fn count(&mut self) -> u8 {
        self.tx.count()
    }
    pub fn flush(&mut self) -> nb::Result<(), core::convert::Infallible> {
        self.tx.flush()
    }
    pub fn write(&mut self, byte: u8) -> nb::Result<(), core::convert::Infallible> {
        self.tx.write(byte)
    }
}

impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

/// Serial transmitter
pub static mut CONSOLE: Console = Console {started:false, tx: Tx {
    _uart: PhantomData,
    _apb_lock: None
}};

/// Macro for sending a formatted string to console (via UART0) for debugging
#[macro_export]
macro_rules! print {
    ($s:expr) => {
        unsafe {
            if $crate::console::CONSOLE.started {
                write!($crate::console::CONSOLE.tx, $s).unwrap();
            }
        }
        // unsafe {$crate::serial:: console::DEBUG_LOG.write_str($s).unwrap()};
    };
    ($($arg:tt)*) => {
        write!($crate::console::CONSOLE.tx, $($arg)*).unwrap();

        // unsafe {$crate::console::DEBUG_LOG.write_fmt(format_args!($($arg)*)).unwrap()};
    };
}

/// Macro for sending a formatted string to the console (via UART0) for debugging, with a newline.
#[macro_export]
macro_rules! println {
    () => {
        unsafe {
            if $crate::console::CONSOLE.started {
                write!($crate::console::CONSOLE.tx, "\n").unwrap();
            }
        }
        // writeln!(tx).unwrap();

        // unsafe {$crate::console::DEBUG_LOG.write_str("\n").unwrap()};
    };
    ($fmt:expr) => {
        unsafe {
            if $crate::console::CONSOLE.started {
                writeln!($crate::console::CONSOLE.tx, $fmt).unwrap();
            }
        }

        // unsafe {$crate::console::DEBUG_LOG.write_str(concat!($fmt, "\n")).unwrap()};
    };
    ($fmt:expr, $($arg:tt)*) => {
        unsafe {
            if $crate::console::CONSOLE.started {
                writeln!($crate::console::CONSOLE.tx, $fmt, $($arg)*).unwrap();
            }
        }
        //unsafe {$crate::console::DEBUG_LOG.write_fmt(format_args!(concat!($fmt, "\n"), $($arg)*)).unwrap()};
    };
}

/// Macro for flushing the console (via UART0).
#[macro_export]
macro_rules! flush {
    () => {
        unsafe {
            if $crate::console::CONSOLE.started {
                $crate::console::CONSOLE.flush().unwrap();
            }
        }
        // unsafe { while !$crate::console::DEBUG_LOG.is_idle() {} };
    };
}
