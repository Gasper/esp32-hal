use embedded_hal::adc::Channel;
use crate::analog::ADC1;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Resolution {
    Resolution9Bit = 0,
    Resolution10Bit = 1,
    Resolution11Bit = 2,
    Resolution12Bit = 3,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Attenuation {
    Attenuation0dB = 0b00,
    Attenuation2p5dB = 0b01,
    Attenuation6dB = 0b10,
    Attenuation11dB = 0b11,
}

pub struct AdcConfig {
    pub resolution: Resolution,
    pub hall_sensor: bool,
    pub attenuations: [Option<Attenuation>; 10],
}

impl AdcConfig{
    pub fn enable_pin<PIN: Channel<ADC1, ID=u8>>(&mut self, _pin: &PIN, attenuation: Attenuation) {
        self.attenuations[PIN::channel() as usize] = Some(attenuation);
    }

    pub fn enable_hall_sensor(&mut self) {
        self.hall_sensor = true;
    }

    pub fn disable_hall_sensor(&mut self) {
        self.hall_sensor = false;
    }
}

impl Default for AdcConfig {
    fn default() -> AdcConfig {
        AdcConfig {
            resolution: Resolution::Resolution12Bit,
            hall_sensor: false,
            attenuations: [None; 10],
        }
    }
}