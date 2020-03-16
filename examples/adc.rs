#![no_std]
#![no_main]
#![feature(asm)]

//use xtensa_lx6_rt as _;
#[macro_use(block)]
extern crate nb;

use core::panic::PanicInfo;

use embedded_hal::adc::OneShot as _;

use esp32;
use esp32_hal::adc::{config::Config, ADC, ADC1};
use esp32_hal::gpio::GpioExt;


#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };
    let all_pins = dp.GPIO.split();
    let mut pin36 = all_pins.gpio36.into_floating_input();
    let mut pin34 = all_pins.gpio34.into_floating_input();

    let mut adc1 = ADC::adc1(Config::default()).unwrap();
    let mut adc1_pin34 = ADC::adc1(Config::default()).unwrap();

    loop {
        let raw_value: u16 = block!(adc1.read(&mut pin36)).unwrap();
        let another_rv: u16 = block!(adc1_pin34.read(&mut pin34)).unwrap();
    }
}

/// Basic panic handler - just loops
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


