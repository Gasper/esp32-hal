use embedded_hal::adc::{Channel, OneShot};
use core::marker::PhantomData;

use crate::pac::{APB_CTRL, SENS};
use crate::gpio::*;


pub struct ADC1;
pub struct ADC2;

pub mod config {
    
    #[derive(PartialEq, Eq, Clone, Copy)]
    pub enum Resolution {
        Resolution9Bit = 0,
        Resolution10Bit = 1,
        Resolution11Bit = 2,
        Resolution12Bit = 3,
    }

    #[derive(PartialEq, Eq)]
    pub enum Attenuation {
        Attenuation0Db = 0b00,
        Attenuation2p5Db = 0b01,
        Attenuation6Db = 0b10,
        Attenuation11Db = 0b11,
    }

    pub struct Config {
        pub resolution: Resolution,
        pub attenuation: Attenuation,
    }

    impl Default for Config {
        fn default() -> Config {
            Config {
                resolution: Resolution::Resolution12Bit,
                attenuation: Attenuation::Attenuation0Db,
            }
        }
    }
}

impl Channel<ADC1> for Gpio36<Input<Floating>> {
    type ID = u8;

    fn channel() -> u8 { 0_u8 }
}

impl Channel<ADC1> for Gpio34<Input<Floating>> {
    type ID = u8;

    fn channel() -> u8 { 0_u8 }
}

pub struct ADC<ADC, PIN> {
    adc: PhantomData<ADC>,
    pin: PhantomData<PIN>,
}

impl<PIN> ADC<ADC1, PIN>
    where PIN: Channel<ADC1, ID=u8> {

    pub fn adc1(config: config::Config) -> Result<Self, ()> {
        let adc = ADC { adc: PhantomData, pin: PhantomData }
            .set_resolution(config.resolution)
            .set_attenuation(config.attenuation);

        Ok(adc)
    }

    pub fn set_resolution(self, resolution: config::Resolution) -> Self {
        let sensors = unsafe { &*SENS::ptr() };
        
        sensors.sar_start_force.modify(|_,w|
            unsafe { w.sar1_bit_width().bits(resolution as u8) }
        );

        sensors.sar_read_ctrl.modify(|_,w|
            unsafe { w.sar1_sample_bit().bits(resolution as u8) }
        );

        self
    }

    pub fn set_attenuation(self, attenuation: config::Attenuation) -> Self {

        let sensors = unsafe { &*SENS::ptr() };
        
        sensors.sar_atten1.modify(|r, w| {
            let new_value = (r.bits() & !(0b11 << (PIN::channel() * 2))) 
                | (((attenuation as u8 & 0b11) as u32) << (PIN::channel() * 2));

            unsafe { w.sar1_atten().bits(new_value) }
        });

        self
    }
}

impl<WORD, PIN> OneShot<ADC1, WORD, PIN> for ADC<ADC1, PIN>
where
   WORD: From<u16>,
   PIN: Channel<ADC1, ID=u8>,
{
    type Error = ();

    fn read(&mut self, _pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        let sensors = unsafe { &*SENS::ptr() };

        // Enable channel
        sensors.sar_meas_start1.modify(|_, w| {
            unsafe { w.sar1_en_pad().bits(1 << PIN::channel() as u8) }
        });

        // Wait for ongoing conversion to complete
        let adc_status = sensors.sar_slave_addr1.read().meas_status().bits() as u8;
        if adc_status != 0 {
            return Err(nb::Error::WouldBlock);
        }

        // Start conversion
        sensors.sar_meas_start1.modify(|_,w| w.meas1_start_sar().clear_bit());
        sensors.sar_meas_start1.modify(|_,w| w.meas1_start_sar().set_bit());

        // Wait for ADC to finish conversion
        let conversion_finished = sensors.sar_meas_start1.read().meas1_done_sar().bit_is_set();
        if !conversion_finished {
            return Err(nb::Error::WouldBlock);
        }

        // Get converted value
        let converted_value = sensors.sar_meas_start1.read().meas1_data_sar().bits() as u16;

        Ok(converted_value.into())
    }
}

