use {
    crate::esp32::{GPIO, IO_MUX, RTCIO},
    core::{convert::Infallible, marker::PhantomData},
    embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin},
};

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
}

/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
pub struct Floating;
/// Pulled down input (type state)
pub struct PullDown;

/// Pulled up input (type state)
pub struct PullUp;

/// Open drain input or output (type state)
pub struct OpenDrain;

/// Push pull output (type state)
pub struct PushPull;



/// Analog mode (type state)
pub struct Analog;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Output mode (type state)
pub struct InputOutput<MODE> {
    _mode: PhantomData<MODE>,
}
/// Alternate function
pub struct Alternate<MODE> {
    _mode: PhantomData<MODE>,
}

/// Alternate Function 1
pub struct AF1;

/// Alternate Function 2
pub struct AF2;

/// Alternate Function 4
pub struct AF4;

/// Alternate Function 5
pub struct AF5;

/// Alternate Function 6
pub struct AF6;

macro_rules! gpio {
    ($GPIO:ident: [
        $($pxi:ident: ($pname:ident, $MODE:ty),)+
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
        )+

    };
}

// All info on reset state pulled from 4.10 IO_MUX Pad List in the reference manual
gpio! {
   GPIO: [
       Gpio0: (gpio0, Input<PullUp>),
       Gpio1: (gpio1, Input<PullUp>),
       Gpio2: (gpio2, Input<PullDown>),
       Gpio3: (gpio3, Input<PullUp>),
       Gpio4: (gpio4, Input<PullDown>),
       Gpio5: (gpio5, Input<PullUp>),
       Gpio6: (gpio6, Input<PullUp>),
       Gpio7: (gpio7, Input<PullUp>),
       Gpio8: (gpio8, Input<PullUp>),
       Gpio9: (gpio9, Input<PullUp>),
       Gpio10: (gpio10, Input<PullUp>),
       Gpio11: (gpio11, Input<PullUp>),
       Gpio12: (gpio12, Input<PullDown>),
       Gpio13: (gpio13, Input<Floating>),
       Gpio14: (gpio14, Input<Floating>),
       Gpio15: (gpio15, Input<PullUp>),
       Gpio16: (gpio16, Input<Floating>),
       Gpio17: (gpio17, Input<Floating>),
       Gpio18: (gpio18, Input<Floating>),
       Gpio19: (gpio19, Input<Floating>),
       Gpio20: (gpio20, Input<Floating>),
       Gpio21: (gpio21, Input<Floating>),
       Gpio22: (gpio22, Input<Floating>),
       Gpio23: (gpio23, Input<Floating>),
       // TODO these pins have a reset mode of 0 (apart from Gpio27),
       // input disable, does that mean they are actually in output mode on reset?
       Gpio25: (gpio25, Input<Floating>),
       Gpio26: (gpio26, Input<Floating>),
       Gpio27: (gpio27, Input<Floating>),
       Gpio32: (gpio32, Input<Floating>),
       Gpio33: (gpio33, Input<Floating>),
       Gpio34: (gpio34, Input<Floating>),
       Gpio35: (gpio35, Input<Floating>),
       Gpio36: (gpio36, Input<Floating>),
       Gpio37: (gpio37, Input<Floating>),
       Gpio38: (gpio38, Input<Floating>),
       Gpio39: (gpio39, Input<Floating>),
   ]
}

macro_rules! impl_output {
    ($en:ident, $outs:ident, $outc:ident, $reg:ident, $reader:ident, [
        // index, gpio_xx pin name, funcX name, iomux pin name
        $($pxi:ident: ($i:expr, $pin:ident, $funcXout:ident, $iomux:ident),)+
    ]) => {
        $(
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
                    Ok(unsafe {& *GPIO::ptr() }.$reg.read().$reader().bits() & (1 << $i) != 0)
                }

                fn is_low(&self) -> Result<bool, Self::Error> {
                    Ok(!self.is_high()?)
                }
            }

            impl<MODE> StatefulOutputPin for $pxi<Output<MODE>> {
                fn is_set_high(&self) -> Result<bool, Self::Error> {
                     // NOTE(unsafe) atomic read to a stateless register
                    unsafe { Ok((*GPIO::ptr()).$outs.read().bits() & (1 << $i) != 0) }
                }

                fn is_set_low(&self) -> Result<bool, Self::Error> {
                    Ok(!self.is_set_high()?)
                }
            }

            impl<MODE> ToggleableOutputPin for $pxi<Output<MODE>> {
                type Error = Infallible;

                fn toggle(&mut self) -> Result<(), Self::Error> {
                    if self.is_set_high()? {
                        Ok(self.set_low()?)
                    } else {
                        Ok(self.set_high()?)
                    }
                }
            }

            impl<MODE> $pxi<MODE> {
                pub fn into_push_pull_output(self) -> $pxi<Output<PushPull>> {
                    let gpio = unsafe{ &*GPIO::ptr() };
                    let iomux = unsafe{ &*IO_MUX::ptr() };
                    self.disable_analog();

                    gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                    gpio.$funcXout.modify(|_, w| unsafe { w.bits(0x100) });

                    iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                    iomux.$iomux.modify(|_, w| w.fun_wpd().set_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpu().set_bit());
                    $pxi { _mode: PhantomData }
                }

                pub fn into_open_drain_output(self) -> $pxi<InputOutput<OpenDrain>> {
                    let gpio = unsafe{ &*GPIO::ptr() };
                    let iomux = unsafe{ &*IO_MUX::ptr() };
                    self.disable_analog();

                    gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                    gpio.$funcXout.modify(|_, w| unsafe { w.bits(0x100) });

                    iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                    iomux.$iomux.modify(|_, w| w.fun_wpd().clear_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpu().clear_bit());
                    $pxi { _mode: PhantomData }
                }

                fn set_alternate(&self, n: u8) {
                    let gpio = unsafe{ &*GPIO::ptr() };
                    let iomux = unsafe{ &*IO_MUX::ptr() };
                    self.disable_analog();

                    gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                    gpio.$funcXout.modify(|_, w| unsafe { w.bits(0x100) });

                    iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(n) });
                    iomux.$iomux.modify(|_, w| w.fun_wpd().clear_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpu().clear_bit());
                }

                pub fn into_alternate_1(self) -> $pxi<Alternate<AF1>> {
                    self.set_alternate(0);
                    $pxi { _mode: PhantomData }
                }

                pub fn into_alternate_2(self) -> $pxi<Alternate<AF2>> {
                    self.set_alternate(1);
                    $pxi { _mode: PhantomData }
                }

                pub fn into_alternate_4(self) -> $pxi<Alternate<AF4>> {
                    self.set_alternate(3);
                    $pxi { _mode: PhantomData }
                }

                pub fn into_alternate_5(self) -> $pxi<Alternate<AF5>> {
                    self.set_alternate(4);
                    $pxi { _mode: PhantomData }
                }

                pub fn into_alternate_6(self) -> $pxi<Alternate<AF6>> {
                    self.set_alternate(5);
                    $pxi { _mode: PhantomData }
                }
            }
        )+
    };
}

impl_output! {
    enable_w1ts, out_w1ts, out_w1tc, in_, in_data, [
        Gpio0: (0, pin0, func0_out_sel_cfg, gpio0),
        Gpio1: (1, pin1, func1_out_sel_cfg, u0txd),
        Gpio2: (2, pin2, func2_out_sel_cfg, gpio2),
        Gpio3: (3, pin3, func3_out_sel_cfg, u0rxd),
        Gpio4: (4, pin4, func4_out_sel_cfg, gpio4),
        Gpio5: (5, pin5, func5_out_sel_cfg, gpio5),
        Gpio6: (6, pin6, func6_out_sel_cfg, sd_clk),
        Gpio7: (7, pin7, func7_out_sel_cfg, sd_data0),
        Gpio8: (8, pin8, func8_out_sel_cfg, sd_data1),
        Gpio9: (9, pin9, func9_out_sel_cfg, sd_data2),
        Gpio10: (10, pin10, func10_out_sel_cfg, sd_data3),
        Gpio11: (11, pin11, func11_out_sel_cfg, sd_cmd),
        Gpio12: (12, pin12, func12_out_sel_cfg, mtdi),
        Gpio13: (13, pin13, func13_out_sel_cfg, mtck),
        Gpio14: (14, pin14, func14_out_sel_cfg, mtms),
        Gpio15: (15, pin15, func15_out_sel_cfg, mtdo),
        Gpio16: (16, pin16, func16_out_sel_cfg, gpio16),
        Gpio17: (17, pin17, func17_out_sel_cfg, gpio17),
        Gpio18: (18, pin18, func18_out_sel_cfg, gpio18),
        Gpio19: (19, pin19, func19_out_sel_cfg, gpio19),
        Gpio20: (20, pin20, func20_out_sel_cfg, gpio20),
        Gpio21: (21, pin21, func21_out_sel_cfg, gpio21),
        Gpio22: (22, pin22, func22_out_sel_cfg, gpio22),
        Gpio23: (23, pin23, func23_out_sel_cfg, gpio23),
        Gpio25: (25, pin25, func25_out_sel_cfg, gpio25),
        Gpio26: (26, pin26, func26_out_sel_cfg, gpio26),
        Gpio27: (27, pin27, func27_out_sel_cfg, gpio27),
    ]
}

impl_output! {
    enable1_w1ts, out1_w1ts, out1_w1tc, in_, in_data, [
        Gpio32: (0, pin32, func32_out_sel_cfg, gpio32),
        Gpio33: (1, pin33, func33_out_sel_cfg, gpio33),
        /* Deliberately omitting 34-39 as these can *only* be inputs */
    ]
}

macro_rules! impl_pullup_pulldown {
    ($en:ident, $pxi:ident, $i:expr, $funcXin:ident,
        $iomux:ident, has_pullup_pulldown) => {
        pub fn into_pull_up_input(self) -> $pxi<Input<PullUp>> {
            let gpio = unsafe{ &*GPIO::ptr() };
            let iomux = unsafe{ &*IO_MUX::ptr() };
            self.disable_analog();

            gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
            gpio.$funcXin.modify(|_, w| unsafe { w.bits(0x100) });

            iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
            iomux.$iomux.modify(|_, w| w.fun_ie().set_bit());
            iomux.$iomux.modify(|_, w| w.fun_wpd().clear_bit());
            iomux.$iomux.modify(|_, w| w.fun_wpu().set_bit());
            $pxi { _mode: PhantomData }
        }

        pub fn into_pull_down_input(self) -> $pxi<Input<PullDown>> {
            let gpio = unsafe{ &*GPIO::ptr() };
            let iomux = unsafe{ &*IO_MUX::ptr() };
            self.disable_analog();

            gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
            gpio.$funcXin.modify(|_, w| unsafe { w.bits(0x100) });

            iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
            iomux.$iomux.modify(|_, w| w.fun_ie().set_bit());
            iomux.$iomux.modify(|_, w| w.fun_wpd().set_bit());
            iomux.$iomux.modify(|_, w| w.fun_wpu().clear_bit());
            $pxi { _mode: PhantomData }
        }
    };
    ($en:ident, $pxi:ident, $i:expr, $funcXin:ident,
        $iomux:ident, no_pullup_pulldown) => {
        /* No pullup/pulldown resistor is available on this pin. */
    };
    ($en:ident, $pxi:ident, $i:expr, $funcXin:ident,
        $iomux:ident, $pullup_flag:ident) => {
        compile_error! ("The GPIO pin has to be marked with either \
            has_pullup_pulldown or no_pullup_pulldown.");
    };
}

macro_rules! impl_input {
    ($en:ident, $reg:ident, $reader:ident [
        // index, gpio_xx pin name, funcX name, iomux pin name, has pullup/down resistors
        $($pxi:ident: ($i:expr, $pin:ident, $funcXin:ident, $iomux:ident, $resistors:ident),)+
    ]) => {
        $(
            impl<MODE> InputPin for $pxi<Input<MODE>> {
                type Error = Infallible;

                fn is_high(&self) -> Result<bool, Self::Error> {
                    Ok(unsafe {& *GPIO::ptr() }.$reg.read().$reader().bits() & (1 << $i) != 0)
                }

                fn is_low(&self) -> Result<bool, Self::Error> {
                    Ok(!self.is_high()?)
                }
            }

            impl<MODE> $pxi<MODE> {
                pub fn into_floating_input(self) -> $pxi<Input<Floating>> {
                    let gpio = unsafe{ &*GPIO::ptr() };
                    let iomux = unsafe{ &*IO_MUX::ptr() };
                    self.disable_analog();

                    gpio.$en.modify(|_, w| unsafe  { w.bits(0x1 << $i) });
                    gpio.$funcXin.modify(|_, w| unsafe { w.bits(0x100) });

                    iomux.$iomux.modify(|_, w| unsafe { w.mcu_sel().bits(0b10) });
                    iomux.$iomux.modify(|_, w| w.fun_ie().set_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpd().clear_bit());
                    iomux.$iomux.modify(|_, w| w.fun_wpu().clear_bit());
                    $pxi { _mode: PhantomData }
                }

                impl_pullup_pulldown! ($en, $pxi, $i, $funcXin, $iomux, $resistors);
            }
        )+
    };
}

impl_input! {
    enable_w1ts, in_, in_data [
        Gpio0: (0, pin0, func0_in_sel_cfg, gpio0, has_pullup_pulldown),
        Gpio1: (1, pin1, func1_in_sel_cfg, u0txd, has_pullup_pulldown),
        Gpio2: (2, pin2, func2_in_sel_cfg, gpio2, has_pullup_pulldown),
        Gpio3: (3, pin3, func3_in_sel_cfg, u0rxd, has_pullup_pulldown),
        Gpio4: (4, pin4, func4_in_sel_cfg, gpio4, has_pullup_pulldown),
        Gpio5: (5, pin5, func5_in_sel_cfg, gpio5, has_pullup_pulldown),
        Gpio6: (6, pin6, func6_in_sel_cfg, sd_clk, has_pullup_pulldown),
        Gpio7: (7, pin7, func7_in_sel_cfg, sd_data0, has_pullup_pulldown),
        Gpio8: (8, pin8, func8_in_sel_cfg, sd_data1, has_pullup_pulldown),
        Gpio9: (9, pin9, func9_in_sel_cfg, sd_data2, has_pullup_pulldown),
        Gpio10: (10, pin10, func10_in_sel_cfg, sd_data3, has_pullup_pulldown),
        Gpio11: (11, pin11, func11_in_sel_cfg, sd_cmd, has_pullup_pulldown),
        Gpio12: (12, pin12, func12_in_sel_cfg, mtdi, has_pullup_pulldown),
        Gpio13: (13, pin13, func13_in_sel_cfg, mtck, has_pullup_pulldown),
        Gpio14: (14, pin14, func14_in_sel_cfg, mtms, has_pullup_pulldown),
        Gpio15: (15, pin15, func15_in_sel_cfg, mtdo, has_pullup_pulldown),
        Gpio16: (16, pin16, func16_in_sel_cfg, gpio16, has_pullup_pulldown),
        Gpio17: (17, pin17, func17_in_sel_cfg, gpio17, has_pullup_pulldown),
        Gpio18: (18, pin18, func18_in_sel_cfg, gpio18, has_pullup_pulldown),
        Gpio19: (19, pin19, func19_in_sel_cfg, gpio19, has_pullup_pulldown),
        Gpio20: (20, pin20, func20_in_sel_cfg, gpio20, has_pullup_pulldown),
        Gpio21: (21, pin21, func21_in_sel_cfg, gpio21, has_pullup_pulldown),
        Gpio22: (22, pin22, func22_in_sel_cfg, gpio22, has_pullup_pulldown),
        Gpio23: (23, pin23, func23_in_sel_cfg, gpio23, has_pullup_pulldown),
        Gpio25: (25, pin25, func25_in_sel_cfg, gpio25, has_pullup_pulldown),
        Gpio26: (26, pin26, func26_in_sel_cfg, gpio26, has_pullup_pulldown),
        Gpio27: (27, pin27, func27_in_sel_cfg, gpio27, has_pullup_pulldown),
    ]
}

impl_input! {
    enable1_w1ts, in1, in1_data [
        Gpio32: (0, pin32, func32_in_sel_cfg, gpio32, has_pullup_pulldown),
        Gpio33: (1, pin33, func33_in_sel_cfg, gpio33, has_pullup_pulldown),
        Gpio34: (2, pin34, func34_in_sel_cfg, gpio34, no_pullup_pulldown),
        Gpio35: (3, pin35, func35_in_sel_cfg, gpio35, no_pullup_pulldown),
        Gpio36: (4, pin36, func36_in_sel_cfg, gpio36, no_pullup_pulldown),
        Gpio37: (5, pin37, func37_in_sel_cfg, gpio37, no_pullup_pulldown),
        Gpio38: (6, pin38, func38_in_sel_cfg, gpio38, no_pullup_pulldown),
        Gpio39: (7, pin39, func39_in_sel_cfg, gpio39, no_pullup_pulldown),
    ]
}

macro_rules! impl_no_analog {
    ([
        $($pxi:ident),+
    ]) => {
        $(
            impl<MODE> $pxi<MODE> {
                #[inline(always)]
                fn disable_analog(&self) {
                    /* No analog functionality on this pin, so nothing to do */
                }
            }
        )+
    };
}

macro_rules! impl_analog {
    ([
        $($pxi:ident: ($i:expr, $pin_reg:ident, $gpio_reg:ident, $mux_sel:ident, $fun_select:ident,
          $pad_driver:ident, $in_enable:ident, $($rue:ident, $rde:ident)?),)+
    ]) => {
        $(
            impl<MODE> $pxi<MODE> {
                pub fn into_analog(self) -> $pxi<Analog> {
                    let rtcio = unsafe{ &*RTCIO::ptr() };

                    rtcio.$pin_reg.modify(|_,w| {
                        // Connect pin to analog / RTC module instead of standard GPIO
                        w.$mux_sel().set_bit();

                        // Select function "RTC function 1" (GPIO) for analog use
                        unsafe { w.$fun_select().bits(0b00) }
                    });

                    // Configure RTC pin as normal output (instead of open drain)
                    rtcio.$gpio_reg.modify(|_,w| w.$pad_driver().clear_bit());

                    // Disable output
                    rtcio.rtc_gpio_enable_w1tc.modify(|_,w| {
                        unsafe { w.rtc_gpio_enable_w1tc().bits(1u32 << $i) }
                    });

                    // Disable input
                    rtcio.$pin_reg.modify(|_,w| w.$in_enable().clear_bit());

                    // Disable pull-up and pull-down resistors on the pin, if it has them
                    $(
                        rtcio.$pin_reg.modify(|_,w| {
                            w.$rue().clear_bit();
                            w.$rde().clear_bit()
                        });
                    )?

                    $pxi { _mode: PhantomData }
                }

                #[inline(always)]
                fn disable_analog(&self) {
                    let rtcio = unsafe{ &*RTCIO::ptr() };
                    rtcio.$pin_reg.modify(|_,w| w.$mux_sel().clear_bit());
                }
            }
        )+
    }
}

impl_no_analog! {[
    Gpio1, Gpio3, Gpio5, Gpio6, Gpio7, Gpio8, Gpio9, Gpio10, Gpio11,
    Gpio16, Gpio17, Gpio18, Gpio19, Gpio20, Gpio21, Gpio22, Gpio23
]}

impl_analog! {[
    Gpio36: (0, rtc_io_sensor_pads, rtc_gpio_pin0, rtc_io_sense1_mux_sel, rtc_io_sense1_fun_sel, rtc_gpio_pin0_pad_driver, rtc_io_sense1_fun_ie,),
    Gpio37: (1, rtc_io_sensor_pads, rtc_gpio_pin1, rtc_io_sense2_mux_sel, rtc_io_sense2_fun_sel, rtc_gpio_pin1_pad_driver, rtc_io_sense2_fun_ie,),
    Gpio38: (2, rtc_io_sensor_pads, rtc_gpio_pin2, rtc_io_sense3_mux_sel, rtc_io_sense3_fun_sel, rtc_gpio_pin2_pad_driver, rtc_io_sense3_fun_ie,),
    Gpio39: (3, rtc_io_sensor_pads, rtc_gpio_pin3, rtc_io_sense4_mux_sel, rtc_io_sense4_fun_sel, rtc_gpio_pin3_pad_driver, rtc_io_sense4_fun_ie,),
    Gpio34: (4, rtc_io_adc_pad, rtc_gpio_pin4, rtc_io_adc1_mux_sel, rtc_io_adc1_fun_sel, rtc_gpio_pin4_pad_driver, rtc_io_adc1_fun_ie,),
    Gpio35: (5, rtc_io_adc_pad, rtc_gpio_pin5, rtc_io_adc2_mux_sel, rtc_io_adc2_fun_sel, rtc_gpio_pin5_pad_driver, rtc_io_adc1_fun_ie,),
    Gpio25: (6, rtc_io_pad_dac1, rtc_gpio_pin6, rtc_io_pdac1_mux_sel, rtc_io_pdac1_fun_sel, rtc_gpio_pin6_pad_driver, rtc_io_pdac1_fun_ie, rtc_io_pdac1_rue, rtc_io_pdac1_rde),
    Gpio26: (7, rtc_io_pad_dac2, rtc_gpio_pin7, rtc_io_pdac2_mux_sel, rtc_io_pdac2_fun_sel, rtc_gpio_pin7_pad_driver, rtc_io_pdac2_fun_ie, rtc_io_pdac2_rue, rtc_io_pdac2_rde),
    Gpio33: (8, rtc_io_xtal_32k_pad, rtc_gpio_pin8, rtc_io_x32n_mux_sel, rtc_io_x32n_fun_sel, rtc_gpio_pin8_pad_driver, rtc_io_x32n_fun_ie, rtc_io_x32n_rue, rtc_io_x32n_rde),
    Gpio32: (9, rtc_io_xtal_32k_pad, rtc_gpio_pin9, rtc_io_x32p_mux_sel, rtc_io_x32p_fun_sel, rtc_gpio_pin9_pad_driver, rtc_io_x32p_fun_ie, rtc_io_x32p_rue, rtc_io_x32p_rde),
    Gpio4:  (10, rtc_io_touch_pad0, rtc_gpio_pin10, rtc_io_touch_pad0_mux_sel, rtc_io_touch_pad0_fun_sel, rtc_gpio_pin10_pad_driver, rtc_io_touch_pad0_fun_ie, rtc_io_touch_pad0_rue, rtc_io_touch_pad0_rde),
    Gpio0:  (11, rtc_io_touch_pad1, rtc_gpio_pin11, rtc_io_touch_pad1_mux_sel, rtc_io_touch_pad1_fun_sel, rtc_gpio_pin11_pad_driver, rtc_io_touch_pad1_fun_ie, rtc_io_touch_pad1_rue, rtc_io_touch_pad1_rde),
    Gpio2:  (12, rtc_io_touch_pad2, rtc_gpio_pin12, rtc_io_touch_pad2_mux_sel, rtc_io_touch_pad2_fun_sel, rtc_gpio_pin12_pad_driver, rtc_io_touch_pad2_fun_ie, rtc_io_touch_pad2_rue, rtc_io_touch_pad2_rde),
    Gpio15: (13, rtc_io_touch_pad3, rtc_gpio_pin13, rtc_io_touch_pad3_mux_sel, rtc_io_touch_pad3_fun_sel, rtc_gpio_pin13_pad_driver, rtc_io_touch_pad3_fun_ie, rtc_io_touch_pad3_rue, rtc_io_touch_pad3_rde),
    Gpio13: (14, rtc_io_touch_pad4, rtc_gpio_pin14, rtc_io_touch_pad4_mux_sel, rtc_io_touch_pad4_fun_sel, rtc_gpio_pin14_pad_driver, rtc_io_touch_pad4_fun_ie, rtc_io_touch_pad4_rue, rtc_io_touch_pad4_rde),
    Gpio12: (15, rtc_io_touch_pad5, rtc_gpio_pin15, rtc_io_touch_pad5_mux_sel, rtc_io_touch_pad5_fun_sel, rtc_gpio_pin15_pad_driver, rtc_io_touch_pad5_fun_ie, rtc_io_touch_pad5_rue, rtc_io_touch_pad5_rde),
    Gpio14: (16, rtc_io_touch_pad6, rtc_gpio_pin16, rtc_io_touch_pad6_mux_sel, rtc_io_touch_pad6_fun_sel, rtc_gpio_pin16_pad_driver, rtc_io_touch_pad6_fun_ie, rtc_io_touch_pad6_rue, rtc_io_touch_pad6_rde),
    Gpio27: (17, rtc_io_touch_pad7, rtc_gpio_pin17, rtc_io_touch_pad7_mux_sel, rtc_io_touch_pad7_fun_sel, rtc_gpio_pin17_pad_driver, rtc_io_touch_pad7_fun_ie, rtc_io_touch_pad7_rue, rtc_io_touch_pad7_rde),
]}