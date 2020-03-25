#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use(block)]
extern crate nb;

use embedded_hal::adc::OneShot;

use xtensa_lx6_rt as _;

use core::fmt::Write;
use core::panic::PanicInfo;
use esp32;
use esp32_hal::gpio::{GpioExt};
use esp32_hal::analog::SensExt;
use esp32_hal::analog::adc::ADC;

use esp32_hal::serial::{NoRx, NoTx, Serial};

use embedded_hal::watchdog::*;

/// The default clock source is the onboard crystal
/// In most cases 40mhz (but can be as low as 2mhz depending on the board)
const CORE_HZ: u32 = 40_000_000;

const BLINK_HZ: u32 = CORE_HZ / 1;

const WDT_WKEY_VALUE: u32 = 0x50D83AA1;

#[no_mangle]
fn main() -> ! {
    let dp = unsafe { esp32::Peripherals::steal() };

    let mut timg0 = dp.TIMG0;
    let mut timg1 = dp.TIMG1;

    // (https://github.com/espressif/openocd-esp32/blob/97ba3a6bb9eaa898d91df923bbedddfeaaaf28c9/src/target/esp32.c#L431)
    // openocd disables the watchdog timer on halt
    // we will do it manually on startup
    disable_timg_wdts(&mut timg0, &mut timg1);

    let mut clkcntrl = esp32_hal::clock_control::ClockControl::new(dp.RTCCNTL, dp.APB_CTRL);
    clkcntrl.watchdog().disable();

    let serial = Serial::uart0(dp.UART0, (NoTx, NoRx), esp32_hal::serial::config::Config::default(), &mut clkcntrl).unwrap();
    let (mut tx, _rx) = serial.split();

    /* Set ADC pins into analog mode */
    let gpios = dp.GPIO.split();
    let mut pin36 = gpios.gpio36.into_analog();
    let mut pin25 = gpios.gpio25.into_analog();

    /* Prepare ADC configs by enabling pins, which will be used */
    let mut adc1_config = esp32_hal::analog::config::Adc1Config::new();
    adc1_config.enable_pin(&pin36, esp32_hal::analog::config::Attenuation::Attenuation11dB);

    let mut adc2_config = esp32_hal::analog::config::Adc2Config::new();
    adc2_config.enable_pin(&pin25, esp32_hal::analog::config::Attenuation::Attenuation11dB);

    /* Create ADC instances */
    let analog = dp.SENS.split();
    let mut adc1 = ADC::adc1(analog.adc1, adc1_config).unwrap();
    let mut adc2 = ADC::adc2(analog.adc2, adc2_config).unwrap();

    loop {
        /* Read ADC values */

        let pin36_value: u16 = block!(adc1.read(&mut pin36)).unwrap();
        writeln!(tx, "ADC1 pin 36 raw value: {:?}", pin36_value).unwrap();

        let pin25_value: u16 = block!(adc2.read(&mut pin25)).unwrap();
        writeln!(tx, "ADC2 pin 25 raw value: {:?}", pin25_value).unwrap();

        delay(BLINK_HZ);
    }
}   

fn disable_timg_wdts(timg0: &mut esp32::TIMG0, timg1: &mut esp32::TIMG1) {
    timg0
        .wdtwprotect
        .write(|w| unsafe { w.bits(WDT_WKEY_VALUE) });
    timg1
        .wdtwprotect
        .write(|w| unsafe { w.bits(WDT_WKEY_VALUE) });

    timg0.wdtconfig0.write(|w| unsafe { w.bits(0x0) });
    timg1.wdtconfig0.write(|w| unsafe { w.bits(0x0) });
}

/// cycle accurate delay using the cycle counter register
pub fn delay(clocks: u32) {
    let start = get_ccount();
    loop {
        if get_ccount().wrapping_sub(start) >= clocks {
            break;
        }
    }
}

/// Performs a special register read to read the current cycle count.
/// In the future, this can be pre-compiled to a archive (.a) and linked to so we don't
/// have to require the asm nightly feature - see cortex-m-rt for more details
pub fn get_ccount() -> u32 {
    let x: u32;
    unsafe { asm!("rsr.ccount a2" : "={a2}"(x) ) };
    x
}

/// Basic panic handler - just loops
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        /*        blinky.set_high().unwrap();
        delay(CORE_HZ/10);
        blinky.set_low().unwrap();
        delay(CORE_HZ/10);
        */
    }
}
