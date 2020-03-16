#![no_std]
#![no_main]
#![feature(asm)]

#[macro_use(block)]
extern crate nb;

use xtensa_lx6_rt as _;

use embedded_hal::adc::OneShot;

use core::fmt::Write;
use core::panic::PanicInfo;
use esp32;
use esp32_hal::gpio::{GpioExt, Gpio36, Input, Floating};
use esp32_hal::hal::digital::v2::OutputPin;
use esp32_hal::adc::{ADC, ADC1};

use esp32_hal::hal::serial::Read as _;

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

    let gpios = dp.GPIO.split();
    let mut pin36 = gpios.gpio36.into_floating_input();

    let mut clkcntrl = esp32_hal::clock_control::ClockControl::new(dp.RTCCNTL, dp.APB_CTRL);
    clkcntrl.watchdog().disable();

    let serial = Serial::uart0(dp.UART0, (NoTx, NoRx), esp32_hal::serial::config::Config::default(), &mut clkcntrl).unwrap();
    let baudrate = serial.get_baudrate();

    let mut adc1: ADC<ADC1, Gpio36<Input<Floating>>> 
        = ADC::adc1(esp32_hal::adc::config::Config::default()).unwrap();

    let (mut tx, mut rx) = serial.split();
    writeln!(tx, "baudrate {:?}", baudrate).unwrap();
    delay(BLINK_HZ);

    loop {
        let adc_result = adc1.read(&mut pin36);
        if adc_result.is_ok() {
            let raw_value: u16 = adc_result.unwrap();
            writeln!(tx, "ADC1 raw value: {:?}", raw_value).unwrap();
        }
        else {
            writeln!(tx, "ADC1 read failed.").unwrap();
        }

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
