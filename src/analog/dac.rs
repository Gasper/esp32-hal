
use core::marker::PhantomData;

use crate::pac::{SENS, RTCIO};
use crate::gpio::*;

pub struct DAC1;
pub struct DAC2;

pub struct DAC<DAC, PIN> {
    _dac: PhantomData<DAC>,
    pin: PIN,
}

// Gpio25<Output<Analog>>
impl DAC<DAC1, u8> {
    pub fn dac1(pin: u8) -> Result<Self, ()> {
        let dac = DAC {
                _dac: PhantomData,
                pin: pin,
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
