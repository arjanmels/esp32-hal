
use core::marker::PhantomData;
use core::convert::Infallible;

use esp32::{GPIO, IO_MUX};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::InputPin;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
}

/// Floating input (type state). This means that neither the pull up nor pull down register is enabled for the pin, there must be an external pull up/down resistor.
pub struct Floating;

/// Pulled down input (type state). This means the pull down register is enabled for the pin.
pub struct PullDown;

/// Pulled up input (type state). This means the pull up register is enabled for the pin.
pub struct PullUp;

/// Open drain input/output (type state). When configured as open drain, the corresponding pin is pulled low (i.e. tied to GND) when set to 0, and open circuited (i.e. not connected) when set to 1.
pub struct OpenDrain;

/// Push pull output (type state). When set to 0, pin is connected to ground, when set to 1, pin is connected to Vdd (+3.3v).
pub struct PushPull;

/// Analog mode (type state)
pub struct Analog;


/// Input mode (type state) - applies to Floating, PullDown and PullUp
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Output mode (type state) - applies to PushPull
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Input Output mode (type state) - applies to OpenDrain
pub struct InputOutput<MODE> {
    _mode: PhantomData<MODE>,
}


/// Generic GPIO type
pub struct Gpio<MODE> {
    /// The GPIO pin number
    pub pin: u8,
    _mode: PhantomData<MODE>,
}

// TODO: implement into_*_output functions for `Gpio`
//
// impl<MODE> OutputPin for Gpio<Output<MODE>> {
//     type Error = Infallible;
//
//     fn set_high(&mut self) -> Result<(), Self::Error> {
//         // NOTE(unsafe) atomic write to a stateless register
//         let gpio_xx = unsafe { &(*GPIO::ptr()) };
//         match self.pin {
//             0..=31 => {
//                 unsafe {
//                     gpio_xx.out_w1ts.write(|w| w.bits(1 << self.pin))
//                 };
//             }
//             32..=33 => {
//                 unsafe {
//                     gpio_xx.out_w1ts.write(|w| w.bits(1 << (self.pin - 32)))
//                 };
//             }
//             _ => unreachable!()
//         }
//         Ok(())
//     }
//
//     fn set_low(&mut self) -> Result<(), Self::Error> {
//         // NOTE(unsafe) atomic write to a stateless register
//         let gpio_xx = unsafe { &(*GPIO::ptr()) };
//         match self.pin {
//             0..=31 => {
//                 unsafe {
//                     gpio_xx.out_w1tc.write(|w| w.bits(1 << self.pin))
//                 };
//             }
//             32..=33 => {
//                 unsafe {
//                     gpio_xx.out1_w1tc.write(|w| w.bits(1 << (self.pin - 32)))
//                 };
//             }
//             _ => unreachable!()
//         }
//         Ok(())
//     }
// }

macro_rules! gpio {
    ($GPIO:ident: [
        $($pxi:ident: ($i:expr, $pname:ident, $MODE:ty),)+
    ]) => {

        impl GpioExt for $GPIO {
            type Parts = Parts;

            fn split(self) -> Self::Parts {
                Parts {
                    $(
                        $pname: $pxi { _mode: PhantomData },
                    )+
                }
            }
        }

        pub struct Parts {
            $(
                /// Pin
                pub $pname: $pxi<$MODE>,
            )+
        }

        // create all the pins, we can also add functionality
        // applicable to all pin states here
        $(
            /// Pin
            pub struct $pxi<MODE> {
                _mode: PhantomData<MODE>,
            }

            impl<MODE> $pxi<MODE> {
                /// Downgrade this pin to a generic Gpio type.
                pub fn downgrade(self) -> Gpio<MODE> {
                    Gpio {
                        pin: $i,
                        _mode: PhantomData,
                    }
                }
            }
        )+
    };
}

// All info on reset state pulled from 4.10 IO_MUX Pad List in the reference manual
// https://www.espressif.com/sites/default/files/documentation/esp32_datasheet_en.pdf
gpio! {
   GPIO: [
       Gpio0: (0, gpio0, Input<PullUp>),
       Gpio1: (1, gpio1, Input<PullUp>),
       Gpio2: (2, gpio2, Input<PullDown>),
       Gpio3: (3, gpio3, Input<PullUp>),
       Gpio4: (4, gpio4, Input<PullDown>),
       Gpio5: (5, gpio5, Input<PullUp>),
       Gpio6: (6, gpio6, Input<PullUp>),
       Gpio7: (7, gpio7, Input<PullUp>),
       Gpio8: (8, gpio8, Input<PullUp>),
       Gpio9: (9, gpio9, Input<PullUp>),
       Gpio10: (10, gpio10, Input<PullUp>),
       Gpio11: (11, gpio11, Input<PullUp>),
       Gpio12: (12, gpio12, Input<PullDown>),
       Gpio13: (13, gpio13, Input<Floating>),
       Gpio14: (14, gpio14, Input<Floating>),
       Gpio15: (15, gpio15, Input<PullUp>),
       Gpio16: (16, gpio16, Input<Floating>),
       Gpio17: (17, gpio17, Input<Floating>),
       Gpio18: (18, gpio18, Input<Floating>),
       Gpio19: (19, gpio19, Input<Floating>),
       Gpio20: (20, gpio20, Input<Floating>),
       Gpio21: (21, gpio21, Input<Floating>),
       Gpio22: (22, gpio22, Input<Floating>),
       Gpio23: (23, gpio23, Input<Floating>),
    //    Gpio24 does not exist.
       // TODO these pins have a reset mode of 0 (apart from Gpio27),
       // input disable, does that mean they are actually in output mode on reset?
       Gpio25: (25, gpio25, Input<Floating>),
       Gpio26: (26, gpio26, Input<Floating>),
       Gpio27: (27, gpio27, Input<Floating>),
    //    Gpio28 does not exist.
    //    Gpio29 does not exist.
    //    Gpio30 does not exist.
    //    Gpio31 does not exist.
       Gpio32: (32, gpio32, Input<Floating>),
       Gpio33: (33, gpio33, Input<Floating>),
       Gpio34: (34, gpio34, Input<Floating>),
       Gpio35: (35, gpio35, Input<Floating>),
       Gpio36: (36, gpio36, Input<Floating>),
       Gpio37: (37, gpio37, Input<Floating>),
       Gpio38: (38, gpio38, Input<Floating>),
       Gpio39: (39, gpio39, Input<Floating>),
   ]
}

macro_rules! impl_gpio_ro {
    ($en:ident, $outs:ident, $outc:ident, $in:ident, $ind:ident, $pxi:ident, $i:expr, $pin:ident, $iomux:ident, $mcu_sel_bits:expr) => {
        impl<MODE> InputPin for $pxi<Input<MODE>> {
            type Error = Infallible;

            fn is_high(&self) -> Result<bool, Self::Error> {
                Ok(unsafe {& *GPIO::ptr() }.$in.read().$ind().bits() & (1 << $i) != 0)
                // Ok(true)
            }

            fn is_low(&self) -> Result<bool, Self::Error> {
                Ok(!self.is_high()?)
            }
        }

        impl<MODE> $pxi<MODE> {
            pub fn into_pull_down(self) -> $pxi<Input<PullDown>> {
                let gpio = unsafe{ &*GPIO::ptr() };
                let iomux = unsafe{ &*IO_MUX::ptr() };
                self.disable_analog();

                gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                get_func_in_sel_cfg!(gpio, $pin).modify(|_, w| unsafe { w.bits(0x100) });

                iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                iomux.$iomux.modify(|_, w| w.fun_ie().set_bit());
                iomux.$iomux.modify(|_, w| w.fun_wpd().set_bit()); // set pull down
                iomux.$iomux.modify(|_, w| w.fun_wpu().clear_bit()); // clear pull up.
                $pxi { _mode: PhantomData }
            }

            pub fn into_pull_up(self) -> $pxi<Input<PullUp>> {
                let gpio = unsafe{ &*GPIO::ptr() };
                let iomux = unsafe{ &*IO_MUX::ptr() };
                self.disable_analog();

                gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                get_func_in_sel_cfg!(gpio, $pin).modify(|_, w| unsafe { w.bits(0x100) });

                iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                iomux.$iomux.modify(|_, w| w.fun_ie().set_bit());
                iomux.$iomux.modify(|_, w| w.fun_wpd().clear_bit()); // clear pull down
                iomux.$iomux.modify(|_, w| w.fun_wpu().set_bit()); // set pull up
                $pxi { _mode: PhantomData }
            }
            pub fn into_floating(self) -> $pxi<Input<Floating>> {
                let gpio = unsafe{ &*GPIO::ptr() };
                let iomux = unsafe{ &*IO_MUX::ptr() };
                self.disable_analog();

                gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                // gpio_xx.$get_func_in_sel_cfg!($pin).modify(|_, w| unsafe { w.bits(0x100) });

                iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                iomux.$iomux.modify(|_, w| w.fun_ie().set_bit());
                iomux.$iomux.modify(|_, w| w.fun_wpd().clear_bit()); // clear pull down
                iomux.$iomux.modify(|_, w| w.fun_wpu().clear_bit()); // clear pull up
                $pxi { _mode: PhantomData }
            }
        }
    }
}
macro_rules! impl_gpio_rw {
    ($en:ident, $outs:ident, $outc:ident, $in:ident, $ind:ident, $pxi:ident, $i:expr, $pin:ident, $iomux:ident, $mcu_sel_bits:expr)
    => {
       impl_gpio_ro!($en, $outs, $outc, $in, $ind, $pxi, $i, $pin, $iomux, $mcu_sel_bits);

       impl<MODE> OutputPin for $pxi<Output<MODE>> {
            type Error = Infallible;

            fn set_high(&mut self) -> Result<(), Self::Error> {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { (*GPIO::ptr()).$outs.write(|w| w.bits(1 << $i)) };
                Ok(())
            }

            fn set_low(&mut self) -> Result<(), Self::Error> {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { (*GPIO::ptr()).$outc.write(|w| w.bits(1 << $i)) };
                Ok(())
            }
        }

        impl<MODE> OutputPin for $pxi<InputOutput<MODE>> {
            type Error = Infallible;

            fn set_high(&mut self) -> Result<(), Self::Error> {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { (*GPIO::ptr()).$outs.write(|w| w.bits(1 << $i)) };
                Ok(())
            }

            fn set_low(&mut self) -> Result<(), Self::Error> {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { (*GPIO::ptr()).$outc.write(|w| w.bits(1 << $i)) };
                Ok(())
            }
        }

        impl<MODE> InputPin for $pxi<InputOutput<MODE>> {
            type Error = Infallible;

            fn is_high(&self) -> Result<bool, Self::Error> {
                // Ok(unsafe {& *GPIO::ptr() }.$reg.read().$reader().bits() & (1 << $i) != 0)
                Ok(true)
            }

            fn is_low(&self) -> Result<bool, Self::Error> {
                Ok(!self.is_high()?)
            }
        }

        impl<MODE> $pxi<MODE> {
            pub fn into_push_pull(self) -> $pxi<Output<PushPull>> {
                let gpio = unsafe{ &*GPIO::ptr() };
                let iomux = unsafe{ &*IO_MUX::ptr() };
                self.disable_analog();

                gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                get_func_out_sel_cfg!(gpio, $pin).modify(|_, w| unsafe { w.bits(0x100) });

                iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                iomux.$iomux.modify(|_, w| w.fun_wpd().set_bit());
                iomux.$iomux.modify(|_, w| w.fun_wpu().set_bit());
                $pxi { _mode: PhantomData }
            }
            /// Go into push/pull mode, but before going into output mode, set the initial value of the value to high. The can help to prevent
            /// an un-desired change on output pin mode configuration.
            pub fn into_push_pull_start_high(self) -> $pxi<Output<PushPull>> {
                // Set the output bit high.
                unsafe { (*GPIO::ptr()).$outs.write(|w| w.bits(1 << $i)) };

                self.into_push_pull()
            }
            /// Go into push/pull mode, but before going into output mode, set the initial value of the value to low. The can help to prevent
            /// an un-desired change on output pin mode configuration.
            pub fn into_push_pull_start_low(self) -> $pxi<Output<PushPull>> {
                // Clear the output bit (set low).
                unsafe { (*GPIO::ptr()).$outc.write(|w| w.bits(1 << $i)) };

                self.into_push_pull()
            }

            /// Go into push/pull mode using an initial state of low as this is open drain.
            pub fn into_open_drain(self) -> $pxi<InputOutput<OpenDrain>> {
                let gpio = unsafe{ &*GPIO::ptr() };
                let iomux = unsafe{ &*IO_MUX::ptr() };
                self.disable_analog();

                gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                get_func_out_sel_cfg!(gpio, $pin).modify(|_, w| unsafe { w.bits(0x100) });

                iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                iomux.$iomux.modify(|_, w| w.fun_wpd().clear_bit());
                iomux.$iomux.modify(|_, w| w.fun_wpu().clear_bit());
                $pxi { _mode: PhantomData }
            }

        }
    }
}



macro_rules! impl_no_adc {
    ($en:ident, $outs:ident, $outc:ident, $pxi:ident, $i:expr, $pin:ident, $iomux:ident, $mcu_sel_bits:expr)
    => {
        impl<MODE> $pxi<MODE> {
            pub fn disable_analog(self) {
                // TODO: No ADC on this pin, so nothing required.
            }
        }
    };
}

macro_rules! impl_has_adc {
    ($en:ident, $outs:ident, $outc:ident, $pxi:ident, $i:expr, $pin:ident, $iomux:ident, $mcu_sel_bits:expr)
    => {
        impl<MODE> $pxi<MODE> {
            pub fn disable_analog(self) {
                // TODO: Implement.
            }
        }
    };
}

macro_rules! impl_no_touch {
    ($en:ident, $outs:ident, $outc:ident, $pxi:ident, $i:expr, $pin:ident, $iomux:ident, $mcu_sel_bits:expr)
    => {
        impl<MODE> $pxi<MODE> {
            #[allow(dead_code)]
            fn dummy() {
                ()
            }
        }
    };

}

macro_rules! impl_has_touch {
    ($en:ident, $outs:ident, $outc:ident, $pxi:ident, $i:expr, $pin:ident, $iomux:ident, $mcu_sel_bits:expr)
    => {
        impl<MODE> $pxi<MODE> {
            #[allow(dead_code)]
            fn dummy() {
                ()
            }
        }
    };

}

macro_rules! impl_gpios {
    ($en:ident, $outs:ident, $outc:ident, $in:ident, $ind:ident, [
    // index, gpio_xx pin name, funcX name, iomux pin name, iomux mcu_sel bits
    $($pxi:ident: ($i:expr, $pin:ident, $iomux:ident, $mcu_sel_bits:expr, $read_or_write_macro:ident, $adc_macro:ident, $capacitive_touch_macro:ident),)+
    ]) => {
        $(
            $read_or_write_macro!($en, $outs, $in, $ind, $outc, $pxi,$i, $pin, $iomux, $mcu_sel_bits);
            $adc_macro!($en, $outs, $outc, $pxi,$i, $pin, $iomux, $mcu_sel_bits);
            $capacitive_touch_macro!($en, $outs, $outc, $pxi,$i, $pin, $iomux, $mcu_sel_bits);
        )+
    };
}

impl_gpios! {
    enable_w1ts, out_w1ts, out_w1tc, in_, in_data, [
        Gpio0: ( 0,  pin0, gpio0, 0b00, impl_gpio_rw, impl_has_adc,impl_has_touch),
        Gpio1: ( 1,  pin1, u0txd, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio2: ( 2,  pin2, gpio2, 0b00, impl_gpio_rw, impl_has_adc,impl_has_touch),
        Gpio3: ( 3,  pin3, u0rxd, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio4: ( 4,  pin4, gpio4, 0b10, impl_gpio_rw, impl_has_adc, impl_has_touch),
        Gpio5: ( 5,  pin5, gpio5, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio6: ( 6,  pin6, sd_clk, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio7: ( 7,  pin7, sd_data0, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio8: ( 8,  pin8, sd_data1, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio9: ( 9,  pin9, sd_data2, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio10: (10, pin10, sd_data3, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio11: (11, pin11, sd_cmd, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio12: (12, pin12, mtdi, 0b10, impl_gpio_rw, impl_has_adc, impl_has_touch),
        Gpio13: (13, pin13, mtck, 0b10, impl_gpio_rw, impl_has_adc, impl_has_touch),
        Gpio14: (14, pin14, mtms, 0b10, impl_gpio_rw, impl_has_adc, impl_has_touch),
        Gpio15: (15, pin15, mtdo, 0b10, impl_gpio_rw, impl_has_adc, impl_has_touch),
        Gpio16: (16, pin16, gpio16, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio17: (17, pin17, gpio17, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio18: (18, pin18, gpio18, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio19: (19, pin19, gpio19, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio20: (20, pin20, gpio20, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio21: (21, pin21, gpio21, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio22: (22, pin22, gpio22, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio23: (23, pin23, gpio23, 0b10, impl_gpio_rw, impl_no_adc,impl_no_touch),
        Gpio25: (25, pin25, gpio25, 0b10, impl_gpio_rw, impl_has_adc,impl_no_touch),
        Gpio26: (26, pin26, gpio26, 0b10, impl_gpio_rw, impl_has_adc,impl_no_touch),
        Gpio27: (27, pin27, gpio27, 0b10, impl_gpio_rw, impl_has_adc,impl_has_touch),
    ]
}

impl_gpios! {
    enable1_w1ts, out1_w1ts, out1_w1tc, in_, in1_data,[
        Gpio32: (0, pin32, gpio32, 0b00, impl_gpio_rw, impl_has_adc, impl_has_touch),
        Gpio33: (1, pin33, gpio33, 0b00, impl_gpio_rw, impl_has_adc, impl_has_touch),
        /* 34-39 can *only* be inputs */
        Gpio34: (2, pin34, gpio34, 0b00, impl_gpio_ro, impl_has_adc, impl_has_touch),
        Gpio35: (3, pin35, gpio35, 0b00, impl_gpio_ro, impl_has_adc, impl_has_touch),
        Gpio36: (4, pin36, gpio36, 0b00, impl_gpio_ro, impl_has_adc, impl_has_touch),
        Gpio37: (5, pin37, gpio37, 0b00, impl_gpio_ro, impl_has_adc, impl_has_touch),
        Gpio38: (6, pin38, gpio38, 0b00, impl_gpio_ro, impl_has_adc, impl_has_touch),
        Gpio39: (7, pin39, gpio39, 0b00, impl_gpio_ro, impl_has_adc, impl_has_touch),
    ]
}

