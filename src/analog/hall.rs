
use embedded_hal::adc::OneShot;
use crate::analog::adc::*;
use crate::pac::{RTCIO};
use crate::gpio::*;

impl ADC<ADC1> {
    pub fn read_hall_sensor(&mut self, vp_pin: &mut Gpio36<Analog>,
        vn_pin: &mut Gpio39<Analog>) -> i32
    {
        let rtcio = unsafe { &*RTCIO::ptr() };

        rtcio.rtc_io_hall_sens.modify(|_,w| w.rtc_io_hall_phase().clear_bit());
        let vp1: u16 = block!(self.read(vp_pin)).unwrap();
        let vn1: u16 = block!(self.read(vn_pin)).unwrap();

        rtcio.rtc_io_hall_sens.modify(|_,w| w.rtc_io_hall_phase().set_bit());
        let vp2: u16 = block!(self.read(vp_pin)).unwrap();
        let vn2: u16 = block!(self.read(vn_pin)).unwrap();

        (vp2 as i32 - vp1 as i32) - (vn2 as i32 - vn1 as i32)
    }
}