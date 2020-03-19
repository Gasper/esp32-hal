#![no_std]
#![no_main]
#![feature(asm)]

use xtensa_lx6_rt as _;

use embedded_hal::adc::OneShot;

use core::fmt::Write;
use core::panic::PanicInfo;
use esp32;
use esp32_hal::gpio::{GpioExt, Gpio36, Input, Floating};
use esp32_hal::hal::digital::v2::OutputPin;
use esp32_hal::analog::dac::{DAC, DAC1};

use esp32_hal::hal::serial::Read as _;

use esp32_hal::serial::{NoRx, NoTx, Serial};

use embedded_hal::watchdog::*;

/// The default clock source is the onboard crystal
/// In most cases 40mhz (but can be as low as 2mhz depending on the board)
const CORE_HZ: u32 = 40_000_000;

const INCREASE_HZ: u32 = CORE_HZ / 4;

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

    let mut clkcntrl = esp32_hal::clock_control::ClockControl::new(dp.RTCCNTL, dp.APB_CTRL);
    clkcntrl.watchdog().disable();

    let mut dac = DAC::dac1(213u8).unwrap();

    let mut voltage: u8 = 0;
    loop {
        voltage = voltage.wrapping_add(1);
        dac.write(voltage);

        delay(INCREASE_HZ);
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
