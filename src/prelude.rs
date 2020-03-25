pub use crate::units::*;

pub use crate::analog::SensExt;
pub use crate::gpio::GpioExt as _esp32_hal_gpio_GpioExt;
pub use crate::units::*;
pub use embedded_hal::adc::OneShot as _esp32_hal_adc_OneShot;
pub use embedded_hal::digital::v2::InputPin as _esp32_hal_digital_InputPin;
pub use embedded_hal::digital::v2::OutputPin as _esp32_hal_digital_OutputPin;
pub use embedded_hal::digital::v2::StatefulOutputPin as _esp32_hal_digital_StatefulOutputPin;
pub use embedded_hal::digital::v2::ToggleableOutputPin as _esp32_hal_digital_ToggleableOutputPin;
pub use embedded_hal::prelude::*;
