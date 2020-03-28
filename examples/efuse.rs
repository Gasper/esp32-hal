#![no_std]
#![no_main]
#![feature(asm)]

use xtensa_lx6_rt as _;

use core::fmt::Write;
use core::panic::PanicInfo;
use esp32;
use esp32_hal::gpio::GpioExt;
use esp32_hal::hal::digital::v2::OutputPin;

use esp32_hal::efuse::Efuse;
use esp32_hal::hal::serial::Read as _;
use esp32_hal::serial::{config::Config, NoRx, NoTx, Serial};
use esp32_hal::units::Hertz;

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
    let mut blinky = gpios.gpio13.into_push_pull_output();

    let mut clkcntrl = esp32_hal::clock_control::ClockControl::new(dp.RTCCNTL, dp.APB_CTRL);
    clkcntrl.watchdog().disable();

    let conf = Config::default().baudrate(Hertz(19_200));
    let serial = Serial::uart0(dp.UART0, (NoTx, NoRx), conf, &mut clkcntrl).unwrap();
    let baudrate = serial.get_baudrate();

    let (mut tx, mut _rx) = serial.split();
    writeln!(tx, "baudrate {:?}", baudrate).unwrap();

    let mac_address = Efuse::get_mac_address();
    let vref: Option<i32> = Efuse::get_adc_vref();
    let adc1_tp_cal: Option<(i32, i32)> = Efuse::get_adc1_two_point_cal();
    let adc2_tp_cal: Option<(i32, i32)> = Efuse::get_adc1_two_point_cal();
    let core_count = Efuse::get_core_count();
    let bt_enabled = Efuse::is_bluetooth_enabled();
    let chip_type = Efuse::get_chip_type();

    /*writeln!(tx, "[1]: {}", mac_addr[1]).unwrap();
    writeln!(tx, "[2]: {}", mac_addr[2]).unwrap();
    writeln!(tx, "[3]: {}", mac_addr[3]).unwrap();
    writeln!(tx, "[4]: {}", mac_addr[4]).unwrap();
    writeln!(tx, "[5]: {}", mac_addr[5]).unwrap();*/

    loop {
        writeln!(tx, "Some data about the chip is here").unwrap();
        writeln!(
            tx,
            "MAC {:#X}:{:#X}:{:#X}:{:#X}:{:#X}:{:#X}",
            mac_address[0],
            mac_address[1],
            mac_address[2],
            mac_address[3],
            mac_address[4],
            mac_address[5]
        )
        .unwrap();
        writeln!(tx, "Bluetooth enabled: {}", bt_enabled).unwrap();
        writeln!(tx, "Chip version: {:?}", chip_type).unwrap();
        writeln!(tx, "Number of CPU cores: {}", core_count).unwrap();

        if let Some(reference_voltage) = vref {
            writeln!(tx, "VREF: {}", reference_voltage).unwrap();
        }

        if let Some((adc1_low, adc1_high)) = adc1_tp_cal {
            writeln!(tx, "ADC1 low: {} ADC1 high: {}", adc1_low, adc1_high).unwrap();
        }

        if let Some((adc2_low, adc2_high)) = adc2_tp_cal {
            writeln!(tx, "ADC2 low: {} ADC2 high: {}", adc2_low, adc2_high).unwrap();
        } else {
            writeln!(tx, "No two-points calibrations").unwrap();
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

/// rough delay - as a guess divide your cycles by 20 (results will differ on opt level)
pub fn delay2(clocks: u32) {
    let dummy_var: u32 = 0;
    for _ in 0..clocks {
        unsafe { core::ptr::read_volatile(&dummy_var) };
    }
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
