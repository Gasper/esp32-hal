#![no_std]
#![no_main]
#![feature(asm)]

use xtensa_lx6_rt as _;

use core::panic::PanicInfo;
use esp32;
use esp32_hal::gpio::GpioExt;
use esp32_hal::analog::SensExt;
use esp32_hal::analog::dac::{DAC};

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
    let pin25 = gpios.gpio25.into_analog();
    let pin26 = gpios.gpio26.into_analog();

    let mut clkcntrl = esp32_hal::clock_control::ClockControl::new(dp.RTCCNTL, dp.APB_CTRL);
    clkcntrl.watchdog().disable();

    let analog = dp.SENS.split();
    let mut dac1 = DAC::dac1(analog.dac1, pin25).unwrap();
    let mut dac2 = DAC::dac2(analog.dac2, pin26).unwrap();

    let mut voltage_dac1: u8 = 0;
    let mut voltage_dac2: u8 = 255;
    loop {
        voltage_dac1 = voltage_dac1.wrapping_add(1);
        dac1.write(voltage_dac1);

        voltage_dac2 = voltage_dac2.wrapping_sub(1);
        dac2.write(voltage_dac2);

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
