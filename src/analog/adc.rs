extern crate spin;

use embedded_hal::adc::{Channel, OneShot};
use core::marker::PhantomData;

use crate::pac::{SENS, RTCIO};
use crate::gpio::*;
use crate::analog::config;

pub struct ADC1;
pub struct ADC2;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AdcError {
    UnconfiguredChannel,
}

pub struct ADC<ADC> {
    adc: PhantomData<ADC>,
    active_channel: spin::Mutex<Option<u8>>,
}

impl ADC<ADC1> {

    pub fn adc1(config: config::AdcConfig) -> Result<Self, ()> {
        let adc = ADC {
                adc: PhantomData,
                active_channel: spin::Mutex::new(None),
            }
            .set_resolution(config.resolution)
            .set_attenuation(config.attenuations)
            .set_controller()
            .set_power()
            .set_hall(config.hall_sensor)
            .set_amp();

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

    pub fn set_attenuation(self, attenuations: [Option<config::Attenuation>; 10]) -> Self {
        let sensors = unsafe { &*SENS::ptr() };
        
        for channel in 0..attenuations.len() {
            if let Some(attenuation) = attenuations[channel] {
                sensors.sar_atten1.modify(|r, w| {
                    let new_value = (r.bits() & !(0b11 << (channel * 2))) 
                        | (((attenuation as u8 & 0b11) as u32) << (channel * 2));
        
                    unsafe { w.sar1_atten().bits(new_value) }
                });
            }
        }

        self
    }

    pub fn set_controller(self) -> Self {
        let sensors = unsafe { &*SENS::ptr() };

        // Set controller to RTC
        sensors.sar_read_ctrl.modify(|_,w| w.sar1_dig_force().clear_bit());
        sensors.sar_meas_start1.modify(|_,w| w.meas1_start_force().set_bit());
        sensors.sar_meas_start1.modify(|_,w| w.sar1_en_pad_force().set_bit());
        sensors.sar_touch_ctrl1.modify(|_,w| w.xpd_hall_force().set_bit());
        sensors.sar_touch_ctrl1.modify(|_,w| w.hall_phase_force().set_bit());

        self
    }

    pub fn set_power(self) -> Self {
        let sensors = unsafe { &*SENS::ptr() };

        // Set power to SW power on
        sensors.sar_meas_wait2.modify(|_,w| {
            unsafe { w.force_xpd_sar().bits(0b11) }
        });

        self
    }

    pub fn set_hall(self, use_hall_sensor: bool) -> Self {
        let rtcio = unsafe { &*RTCIO::ptr() };

        if use_hall_sensor {
            rtcio.rtc_io_hall_sens.modify(|_,w| w.rtc_io_xpd_hall().set_bit());
        }
        else {
            rtcio.rtc_io_hall_sens.modify(|_,w| w.rtc_io_xpd_hall().clear_bit());
        }

        self
    }

    pub fn set_amp(self) -> Self {
        let sensors = unsafe { &*SENS::ptr() };

        // AMP disable - undocumented
        sensors.sar_meas_wait2.modify(|_,w| {
            unsafe { w.force_xpd_amp().bits(0b10) }
        });
        sensors.sar_meas_ctrl.modify(|_,w| unsafe { w.amp_rst_fb_fsm().bits(0) });
        sensors.sar_meas_ctrl.modify(|_,w| unsafe { w.amp_short_ref_fsm().bits(0) });
        sensors.sar_meas_ctrl.modify(|_,w| unsafe { w.amp_short_ref_gnd_fsm().bits(0) });
        sensors.sar_meas_wait1.modify(|_,w| unsafe { w.sar_amp_wait1().bits(1) });
        sensors.sar_meas_wait1.modify(|_,w| unsafe { w.sar_amp_wait2().bits(1) });
        sensors.sar_meas_wait2.modify(|_,w| unsafe { w.sar_amp_wait3().bits(1) });

        self
    }
}

impl<WORD, PIN> OneShot<ADC1, WORD, PIN> for ADC<ADC1>
where
   WORD: From<u16>,
   PIN: Channel<ADC1, ID=u8>,
{
    type Error = AdcError;

    fn read(&mut self, _pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        let sensors = unsafe { &*SENS::ptr() };

        // TODO: reject channels which are not configured

        let active_lock = self.active_channel.try_lock();
        if active_lock.is_none() {
            // Some other thread is calling this function - wait for them to finish
            return Err(nb::Error::WouldBlock);
        }

        let mut current_conversion = active_lock.unwrap();
        if let Some(active_channel) = *current_conversion {
            // There is conversion in progress:
            // - if it's for a different channel try again later
            // - if it's for the given channel, go ahaid and check progress
            if active_channel != PIN::channel() {
                return Err(nb::Error::WouldBlock);
            }
        }
        else {
            // If no conversions are in progress, start a new one for given channel
            *current_conversion = Some(PIN::channel());

            sensors.sar_meas_start1.modify(|_, w| {
                unsafe { w.sar1_en_pad().bits(1 << PIN::channel() as u8) }
            });
    
            sensors.sar_meas_start1.modify(|_,w| w.meas1_start_sar().clear_bit());
            sensors.sar_meas_start1.modify(|_,w| w.meas1_start_sar().set_bit());    
        }

        // Wait for ongoing conversion to complete
        /*let adc_status = sensors.sar_slave_addr1.read().meas_status().bits() as u8;
        if adc_status != 0 {
            return Err(nb::Error::WouldBlock);
        }*/

        // Wait for ADC to finish conversion
        let conversion_finished = sensors.sar_meas_start1.read().meas1_done_sar().bit_is_set();
        if !conversion_finished {
            return Err(nb::Error::WouldBlock);
        }

        // Get converted value
        let converted_value = sensors.sar_meas_start1.read().meas1_data_sar().bits() as u16;

        // Mark that no conversions are currently in progress 
        *current_conversion = None;
        
        Ok(converted_value.into())
    }
}

macro_rules! impl_adc {
    ($adc:ident: [
        $( ($pin:ident, $channel:expr) ,)+
    ]) => {
        $(
            impl Channel<$adc> for $pin<Analog> {
                type ID = u8;
            
                fn channel() -> u8 { $channel }
            }
        )+
    }
}

impl_adc! {
    ADC1: [
        (Gpio36, 0), // Alt. name: SENSOR_VP
        (Gpio37, 1), // Alt. name: SENSOR_CAPP
        (Gpio38, 2), // Alt. name: SENSOR_CAPN
        (Gpio39, 3), // Alt. name: SENSOR_VN
        (Gpio33, 4), // Alt. name: 32K_XP
        (Gpio32, 5), // Alt. name: 32K_XN
        (Gpio34, 6), // Alt. name: VDET_1
        (Gpio35, 7), // Alt. name: VDET_2
    ]
}

impl_adc! {
    ADC2: [
        (Gpio4, 0),
        (Gpio0, 1),
        (Gpio2, 2),
        (Gpio15, 3), // Alt. name: MTDO
        (Gpio13, 4), // Alt. name: MTCK
        (Gpio12, 5), // Alt. name: MTDI
        (Gpio14, 6), // Alt. name: MTMS
        (Gpio27, 7),
        (Gpio25, 8),
        (Gpio26, 9),
    ]
}
