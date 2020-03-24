
use core::marker::PhantomData;

use crate::analog::{DAC1, DAC2};
use crate::pac::{SENS, RTCIO};
use crate::gpio::*;

pub struct DAC<DAC, PIN> {
    _dac: PhantomData<DAC>,
    _pin: PhantomData<PIN>,
}

impl DAC<DAC1, Gpio25<Analog>> {
    pub fn dac1(_dac: DAC1, _pin: Gpio25<Analog>) -> Result<Self, ()> {
        let dac = DAC::<DAC1, Gpio25<Analog>> {
                _dac: PhantomData,
                _pin: PhantomData,
            }
            .set_power();

        Ok(dac)
    }

    fn set_power(self) -> Self {
        let rtcio = unsafe { &*RTCIO::ptr() };

        rtcio.rtc_io_pad_dac1.modify(|_,w| {
            w.rtc_io_pdac1_dac_xpd_force().set_bit();
            w.rtc_io_pdac1_xpd_dac().set_bit()
        });

        self
    }

    pub fn write(&mut self, value: u8) {
        let rtcio = unsafe { &*RTCIO::ptr() };
        let sensors = unsafe { &*SENS::ptr() };

        sensors.sar_dac_ctrl2.modify(|_,w| w.dac_cw_en1().clear_bit());
        rtcio.rtc_io_pad_dac1.modify(|_,w| {
            unsafe { w.rtc_io_pdac1_dac().bits(value) }
        });
    }
}

impl DAC<DAC2, Gpio26<Analog>> {
    pub fn dac2(_dac: DAC2, _pin: Gpio26<Analog>) -> Result<Self, ()> {
        let dac = DAC::<DAC2, Gpio26<Analog>> {
                _dac: PhantomData,
                _pin: PhantomData,
            }
            .set_power();

        Ok(dac)
    }

    fn set_power(self) -> Self {
        let rtcio = unsafe { &*RTCIO::ptr() };

        rtcio.rtc_io_pad_dac2.modify(|_,w| {
            w.rtc_io_pdac2_dac_xpd_force().set_bit();
            w.rtc_io_pdac2_xpd_dac().set_bit()
        });

        self
    }

    pub fn write(&mut self, value: u8) {
        let rtcio = unsafe { &*RTCIO::ptr() };
        let sensors = unsafe { &*SENS::ptr() };

        sensors.sar_dac_ctrl2.modify(|_,w| w.dac_cw_en2().clear_bit());
        rtcio.rtc_io_pad_dac2.modify(|_,w| {
            unsafe { w.rtc_io_pdac2_dac().bits(value) }
        });
    }
}