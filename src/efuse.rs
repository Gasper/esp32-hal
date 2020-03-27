#![allow(non_camel_case_types)]

use esp32::EFUSE;

pub struct Efuse;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ChipType {
    ESP32D0WDQ6,
    ESP32D0WDQ5,
    ESP32D2WDQ5,
    ESP32PICOD2,
    ESP32PICOD4,
    Unknown,
}

impl Efuse {

    pub fn get_mac_address() -> [u8; 6] {
        let efuse = unsafe{ &*EFUSE::ptr() };

        let mac_low: u32 = efuse.blk0_rdata1.read().rd_wifi_mac_crc_low().bits();
        let mac_high: u32 = efuse.blk0_rdata2.read().rd_wifi_mac_crc_high().bits();

        let mac_low_bytes = mac_low.to_be_bytes();
        let mac_high_bytes = mac_high.to_be_bytes();

        [
            mac_high_bytes[2],
            mac_high_bytes[3],
            mac_low_bytes[0],
            mac_low_bytes[1],
            mac_low_bytes[2],
            mac_low_bytes[3],
        ]
    }

    pub fn get_core_count() -> u8 {
        let efuse = unsafe{ &*EFUSE::ptr() };

        // Check if app CPU is disabled
        let cpu_disabled = efuse.blk0_rdata3.read().rd_chip_ver_dis_app_cpu().bit();
        if cpu_disabled {
            1
        }
        else {
            2
        }
    }

    pub fn get_max_cpu_fequency() -> u8 {
        let efuse = unsafe{ &*EFUSE::ptr() };

        let has_rating = efuse.blk0_rdata3.read().rd_chip_cpu_freq_rated().bit();
        let has_low_rating = efuse.blk0_rdata3.read().rd_chip_cpu_freq_low().bit();

        if has_rating && has_low_rating {
            160
        }
        else {
            240
        }
    }

    pub fn is_bluetooth_enabled() -> bool {
        let efuse = unsafe{ &*EFUSE::ptr() };

        !efuse.blk0_rdata3.read().rd_chip_ver_dis_bt().bit()
    }

    pub fn get_chip_version() -> ChipType {
        let efuse = unsafe{ &*EFUSE::ptr() };

        match efuse.blk0_rdata3.read().rd_chip_ver_pkg().bits() {
            0 => ChipType::ESP32_D0WDQ6,
            1 => ChipType::ESP32_D0WDQ5,
            2 => ChipType::ESP32_D2WDQ5,
            4 => ChipType::ESP32_PICOD2,
            5 => ChipType::ESP32_PICOD4,
            _ => ChipType::Unknown,
        }
    }

    pub fn get_adc_vref() -> Option<i32> {
        let efuse = unsafe{ &*EFUSE::ptr() };

        // The base voltage of this calibration value is 1.1V
        let base_voltage: i32 = 1100;
        // The calibration is given as offset in 7mV steps
        let step: i32 = 7;

        let calibration = efuse.blk0_rdata4.read().rd_adc_vref().bits();
        if calibration == 0 {
            return None;
        }

        // The calibration in the register is given as 5 bits:
        // <sign: 1bit> <offset: 4 bit>
        // NOTE: on some older chips this value was given as two's complement
        let (sign, offset) = ((calibration >> 4) as i32, (calibration & 0x0F) as i32);

        if sign == 0 {
            Some(base_voltage + (offset * step))
        }
        else {
            Some(base_voltage - (offset * step))
        }
    }

    pub fn get_adc1_two_point_cal() -> Option<(i32, i32)> {
        let efuse = unsafe{ &*EFUSE::ptr() };

        // Low point represents ADC's raw reading at 150mV.
        // It is given as a 7-bit number, in two's complement,
        // which represents number of steps of 4 (no units, as this
        // is raw ADC value). The offset is applied to predefined
        // base (e.g. 278).

        // Similar is true for high point, except that it's ADC's
        // reading at 850mV, and it's a 9-bit number.

        let adc1_low_base: i32 = 278;
        let adc1_high_base: i32 = 3265;
        let adc1_step: i32 = 4;
        let adc2_low_bit_size = 7;
        let adc2_high_bit_size = 9;

        let adc1_low = efuse.blk3_rdata3.read().rd_adc1_tp_low().bits() as u16;
        let adc1_high = efuse.blk3_rdata3.read().rd_adc1_tp_high().bits() as u16;

        if adc1_low == 0 || adc1_high == 0 {
            None
        }
        else {
            Some((
                adc1_low_base + Efuse::from_twos_complement(adc1_low, adc2_low_bit_size) * adc1_step,
                adc1_high_base + Efuse::from_twos_complement(adc1_high, adc2_high_bit_size) * adc1_step
            ))
        }
    }

    pub fn get_adc2_two_point_cal() -> Option<(i32, i32)> {
        let efuse = unsafe{ &*EFUSE::ptr() };

        let adc2_low_base: i32 = 421;
        let adc2_high_base: i32 = 3406;
        let adc2_step: i32 = 4;
        let adc2_low_bit_size = 7;
        let adc2_high_bit_size = 9;

        let adc2_low = efuse.blk3_rdata3.read().rd_adc2_tp_low().bits() as u16;
        let adc2_high = efuse.blk3_rdata3.read().rd_adc2_tp_high().bits() as u16;

        if adc2_low == 0 || adc2_high == 0 {
            None
        }
        else {
            Some((
                adc2_low_base + Efuse::from_twos_complement(adc2_low, adc2_low_bit_size) * adc2_step,
                adc2_high_base + Efuse::from_twos_complement(adc2_high, adc2_high_bit_size) * adc2_step
            ))
        }
    }

    fn from_twos_complement(value: u16, bits: u8) -> i32 {
        let mask = 2_u16.pow(bits as u32 - 1) - 1;
        let complement_value = (!(value & mask) + 1) as i32;

        let sign_bit = (value >> bits - 1) & 0x01;
        if sign_bit == 0 {
            complement_value
        }
        else {
            -complement_value
        }
    }
}
