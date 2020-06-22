use {
    crate::{
        dprintln,
        esp32::{i2c, DPORT, GPIO, I2C0, I2C1, IO_MUX},
        prelude::*,
    },
    core::ops::Deref,
    embedded_hal::digital::v2::OutputPin,
};

unsafe fn gpio_matrix_out(gpio: u32, signal_idx: u32, out_inverted: bool, oen_inverted: bool) {
    let base_address = 0x3FF44530; // GPIO_FUNC0_OUT_SEL_CFG_REG
    let store_address = (base_address + 4 * gpio) as *mut u32;

    let mut value = signal_idx;

    if out_inverted {
        value = value | 0b100_0000_0000;
    }

    if oen_inverted {
        value = value | 0b1_0000_0000_0000;
    }

    core::ptr::write_volatile(store_address, value);
}

unsafe fn gpio_matrix_in(gpio: u32, signal_idx: u32, inverted: bool) {
    let base_address = 0x3FF44130; // GPIO_FUNC0_IN_SEL_CFG_REG
    let store_address = (base_address + 4 * signal_idx) as *mut u32;

    let mut value = gpio;

    if inverted {
        value = value | 64;
    }

    if gpio != 52 {
        value = value | 128;
    }

    core::ptr::write_volatile(store_address, value);
}

pub struct I2C<T>(T);

impl<T> I2C<T>
where
    T: Instance,
{
    pub fn new<
        SDA: OutputPin<Error = core::convert::Infallible>,
        SCL: OutputPin<Error = core::convert::Infallible>,
    >(
        i2c: T,
        mut pins: Pins<SDA, SCL>,
        dport: &mut DPORT,
    ) -> Self {
        let mut i2c = Self(i2c);

        // i2c_set_pin
        // TEMPORARY UNTIL I CORRECTLY IMPLEMENT PINS
        // manually set sda to gpio4 and scl to gpio15
        unsafe {
            let gpio = &*GPIO::ptr();
            let iomux = &*IO_MUX::ptr();

            // sda
            {
                pins.sda.set_high().unwrap();
                // gpio_set_level(sda_io_num, I2C_IO_INIT_LEVEL);
                gpio.pin4.write(|w| w.pin4_pad_driver().set_bit());

                // PIN_FUNC_SELECT(GPIO_PIN_MUX_REG[sda_io_num], PIN_FUNC_GPIO);
                // gpio_set_direction(sda_io_num, GPIO_MODE_INPUT_OUTPUT_OD);
                // gpio_set_pull_mode(sda_io_num, GPIO_PULLUP_ONLY);
                iomux.gpio4.modify(|_, w| {
                    w.mcu_sel()
                        .bits(2)
                        .fun_wpd()
                        .clear_bit()
                        .fun_wpu()
                        .set_bit()
                        .fun_ie()
                        .set_bit()
                        .mcu_oe()
                        .set_bit()
                });

                gpio_matrix_out(4, 30, false, false);
                gpio_matrix_in(4, 30, false);
            }

            // scl
            {
                pins.scl.set_high().unwrap();
                gpio.pin15.write(|w| w.pin15_pad_driver().set_bit());

                // PIN_FUNC_SELECT(GPIO_PIN_MUX_REG[scl_io_num], PIN_FUNC_GPIO);
                // gpio_set_direction(scl_io_num, GPIO_MODE_INPUT_OUTPUT_OD);
                // gpio_set_pull_mode(scl_io_num, GPIO_PULLUP_ONLY);
                iomux.mtdo.modify(|_, w| {
                    w.mcu_sel()
                        .bits(2)
                        .fun_ie()
                        .set_bit()
                        .mcu_oe()
                        .set_bit()
                        .fun_wpd()
                        .clear_bit()
                        .fun_wpu()
                        .set_bit()
                });

                gpio_matrix_out(15, 29, false, false);
                gpio_matrix_in(15, 29, false);
            }
        }

        // i2c_hw_enable(i2c_num);
        i2c.reset(dport);
        i2c.enable(dport);

        // i2c_hal_disable_intr_mask(&(i2c_context[i2c_num].hal), 0x3FFF);
        i2c.0.int_ena.write(|w| unsafe { w.bits(0) });
        // i2c_hal_clr_intsts_mask(&(i2c_context[i2c_num].hal), 0x3FFF);
        i2c.0.int_clr.write(|w| unsafe { w.bits(0x3FFF) });

        //i2c_ll_master_init(hal->dev);
        //MSB
        //i2c_ll_set_data_mode(hal->dev, I2C_DATA_MODE_MSB_FIRST, I2C_DATA_MODE_MSB_FIRST);
        i2c.0.ctr.modify(|_, w| unsafe {
            w.bits(0)
                .ms_mode()
                .set_bit()
                .sda_force_out()
                .set_bit()
                .scl_force_out()
                .set_bit()
                .tx_lsb_first()
                .clear_bit()
                .rx_lsb_first()
                .clear_bit()
        });

        //i2c_ll_set_fifo_mode(hal->dev, true);
        i2c.0.fifo_conf.modify(|_, w| w.nonfifo_en().clear_bit());

        i2c.reset_fifo();

        i2c.set_filter(Some(7), Some(7));

        i2c.set_frequency(200_000);

        i2c.0.ctr.modify(|_, w| w.clk_en().set_bit());

        i2c
    }

    /// Resets the interface
    fn reset(&mut self, dport: &mut DPORT) {
        dport.perip_rst_en.modify(|_, w| w.i2c0().set_bit());
        dport.perip_rst_en.modify(|_, w| w.i2c0().clear_bit());
    }

    /// Enables the interface
    fn enable(&mut self, dport: &mut DPORT) {
        dport.perip_clk_en.modify(|_, w| w.i2c0().set_bit());
        dport.perip_rst_en.modify(|_, w| w.i2c0().clear_bit());
    }

    /// Resets the transmit and receive FIFO buffers
    fn reset_fifo(&mut self) {
        //i2c_ll_txfifo_rst(hal->dev);
        self.0.fifo_conf.modify(|_, w| w.tx_fifo_rst().set_bit());
        self.0.fifo_conf.modify(|_, w| w.tx_fifo_rst().clear_bit());
        //i2c_ll_rxfifo_rst(hal->dev);
        self.0.fifo_conf.modify(|_, w| w.rx_fifo_rst().set_bit());
        self.0.fifo_conf.modify(|_, w| w.rx_fifo_rst().clear_bit());
    }

    /// Sets the filter with a supplied threshold in clock cycles for which a pulse must be present to pass the filter
    fn set_filter(&mut self, sda_threshold: Option<u8>, scl_threshold: Option<u8>) {
        // i2c_hal_set_filter(&(i2c_context[i2c_num].hal), 7);

        match sda_threshold {
            Some(threshold) => {
                self.0
                    .sda_filter_cfg
                    .modify(|_, w| unsafe { w.sda_filter_thres().bits(threshold) });
                self.0
                    .sda_filter_cfg
                    .modify(|_, w| w.sda_filter_en().set_bit());
            }
            None => self
                .0
                .sda_filter_cfg
                .modify(|_, w| w.sda_filter_en().clear_bit()),
        }

        match scl_threshold {
            Some(threshold) => {
                self.0
                    .scl_filter_cfg
                    .modify(|_, w| unsafe { w.scl_filter_thres().bits(threshold) });
                self.0
                    .scl_filter_cfg
                    .modify(|_, w| w.scl_filter_en().set_bit());
            }
            None => self
                .0
                .scl_filter_cfg
                .modify(|_, w| w.scl_filter_en().clear_bit()),
        }
    }

    /// Sets the freqency of the I2C interface by calculating and applying the associated timings
    fn set_frequency(&mut self, freq: u32) {
        // i2c_hal_set_bus_timing(&(i2c_context[i2c_num].hal), freq, 1);
        // i2c_ll_cal_bus_clk(80000000, freq, 0);
        let half_cycle = ((80_000_000 / freq) / 2) as u16;
        let scl_low = half_cycle;
        let scl_high = half_cycle;
        let sda_hold = half_cycle / 2;
        let sda_sample = scl_high / 2;
        let setup = half_cycle;
        let hold = half_cycle;
        let tout = half_cycle * 20; // By default we set the timeout value to 10 bus cycles.

        // i2c_ll_set_bus_timing(hal->dev, 0);
        unsafe {
            // scl period
            self.0.scl_low_period.write(|w| w.period().bits(scl_low));
            self.0.scl_high_period.write(|w| w.period().bits(scl_high));

            // sda sample
            self.0.sda_hold.write(|w| w.time().bits(sda_hold));
            self.0.sda_sample.write(|w| w.time().bits(sda_sample));

            // setup
            self.0.scl_rstart_setup.write(|w| w.time().bits(setup));
            self.0.scl_stop_setup.write(|w| w.time().bits(setup));

            // hold
            self.0.scl_start_hold.write(|w| w.time().bits(hold));
            self.0.scl_stop_hold.write(|w| w.time().bits(hold));

            // timeout
            self.0.to.write(|w| w.time_out_reg().bits(tout.into()));
        }
    }

    pub fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        dprintln!("addr: {:?}, bytes: {:?}", addr, &bytes);

        // TODO: Use bytes.chunk(255) to remove this limitation
        assert!(bytes.len() < 255);

        // Address for I2C0 (obviously this shouldn't make it into the HAL)
        let fifo_addr = 0x6001301c as *mut u8;

        // Clear FIFO
        self.0.fifo_conf.modify(|_, w| w.tx_fifo_rst().set_bit());
        self.0.fifo_conf.modify(|_, w| w.tx_fifo_rst().clear_bit());
        self.0.fifo_conf.modify(|_, w| w.rx_fifo_rst().set_bit());
        self.0.fifo_conf.modify(|_, w| w.rx_fifo_rst().clear_bit());

        // RSTART command
        self.0.comd0.write(|w| unsafe { w.command0().bits(0) });

        // Address byte
        unsafe {
            core::ptr::write_volatile(fifo_addr, addr << 1 | 0);
        }
        // Data bytes
        for byte in bytes {
            unsafe {
                core::ptr::write_volatile(fifo_addr, *byte);
            }
        }

        // WRITE command
        self.0.comd1.write(|w| unsafe {
            w.command1()
                .bits(0b00_1100_0000_0000 | (1 + bytes.len() as u8) as u16)
        });

        // STOP command
        self.0
            .comd2
            .write(|w| unsafe { w.command2().bits(0b01_1000_0000_0000) });

        dprintln!("txfifo_cnt: {:?}", self.0.sr.read().txfifo_cnt().bits());

        // Start transmission
        self.0.ctr.modify(|_, w| w.trans_start().set_bit());

        while self.0.comd0.read().command0_done().bit() != true {}
        dprintln!("start");
        while self.0.comd1.read().command1_done().bit() != true {}
        dprintln!("write");
        while self.0.comd2.read().command2_done().bit() != true {}
        dprintln!("stop");

        Ok(())
    }

    pub fn read(&mut self, _addr: u8, _bytes: &mut [u8]) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn write_then_read(
        &mut self,
        _addr: u8,
        _bytes: &[u8],
        _buffer: &mut [u8],
    ) -> Result<(), Error> {
        unimplemented!()
    }

    /// Return the raw interface to the underlying I2C peripheral
    pub fn free(self) -> T {
        self.0
    }
}

/// Implementation of embedded_hal::blocking::i2c Traits

impl<T> embedded_hal::blocking::i2c::Write for I2C<T>
where
    T: Instance,
{
    type Error = Error;

    fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Error> {
        self.write(addr, bytes)
    }
}

impl<T> embedded_hal::blocking::i2c::Read for I2C<T>
where
    T: Instance,
{
    type Error = Error;

    fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Error> {
        self.read(addr, bytes)
    }
}

impl<T> embedded_hal::blocking::i2c::WriteRead for I2C<T>
where
    T: Instance,
{
    type Error = Error;

    fn write_read<'w>(
        &mut self,
        addr: u8,
        bytes: &'w [u8],
        buffer: &'w mut [u8],
    ) -> Result<(), Error> {
        self.write_then_read(addr, bytes, buffer)
    }
}

/// Pins used by the I2C interface
///
/// Note that any two pins may be used
/// TODO: enforce this in the type system
pub struct Pins<
    SDA: embedded_hal::digital::v2::OutputPin,
    SCL: embedded_hal::digital::v2::OutputPin,
> {
    pub sda: SDA,
    pub scl: SCL,
}

#[derive(Debug)]
pub enum Error {
    Transmit,
    Receive,
}

pub trait Instance: Deref<Target = i2c::RegisterBlock> {}

impl Instance for I2C0 {}

impl Instance for I2C1 {}