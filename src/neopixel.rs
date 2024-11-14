use core::time::Duration;
use error_stack::ResultExt;
use esp_idf_svc::hal::gpio::Gpio8;
use esp_idf_svc::hal::rmt::config::TransmitConfig;
use esp_idf_svc::hal::rmt::{
    FixedLengthSignal, PinState, Pulse, RmtChannel, TxRmtDriver, CHANNEL0,
};
use thiserror::Error;

pub struct Neopixel {
    driver: TxRmtDriver<'static>,
}

impl Neopixel {
    pub fn new(pin: Gpio8, rmt: CHANNEL0) -> Self {
        let config = TransmitConfig::new().clock_divider(1);
        let driver = TxRmtDriver::new(rmt, pin, &config).unwrap();

        Self { driver }
    }

    pub fn set_color(&mut self, rgb: Rgb) -> error_stack::Result<(), SetNeopixelColorError> {
        let color: u32 = rgb.into();
        let ticks_hz = self
            .driver
            .counter_clock()
            .change_context(SetNeopixelColorError)?;
        let (t0h, t0l, t1h, t1l) = (
            Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))
                .change_context(SetNeopixelColorError)?,
            Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))
                .change_context(SetNeopixelColorError)?,
            Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))
                .change_context(SetNeopixelColorError)?,
            Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))
                .change_context(SetNeopixelColorError)?,
        );
        let mut signal = FixedLengthSignal::<24>::new();
        for i in (0..24).rev() {
            let p = 2_u32.pow(i);
            let bit: bool = p & color != 0;
            let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
            signal
                .set(23 - i as usize, &(high_pulse, low_pulse))
                .change_context(SetNeopixelColorError)?;
        }
        self.driver
            .start(signal)
            .change_context(SetNeopixelColorError)?;
        //self.driver.start_blocking(&signal).change_context(SetNeopixelColorError)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("set neopixel color")]
pub struct SetNeopixelColorError;

pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Error)]
#[error("The given HSV values are not in valid range")]
pub struct FromHsvError;

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    /// Converts hue, saturation, value to RGB
    pub fn from_hsv(h: u32, s: u32, v: u32) -> Result<Self, FromHsvError> {
        if h > 360 || s > 100 || v > 100 {
            return Err(FromHsvError);
        }
        let s = s as f64 / 100.0;
        let v = v as f64 / 100.0;
        let c = s * v;
        let x = c * (1.0 - (((h as f64 / 60.0) % 2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = match h {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        Ok(Self {
            r: ((r + m) * 255.0) as u8,
            g: ((g + m) * 255.0) as u8,
            b: ((b + m) * 255.0) as u8,
        })
    }
}

impl From<u32> for Rgb {
    fn from(color: u32) -> Self {
        Self {
            r: ((color >> 16) & 0xFF) as u8,
            g: ((color >> 8) & 0xFF) as u8,
            b: (color & 0xFF) as u8,
        }
    }
}

impl From<i32> for Rgb {
    fn from(color: i32) -> Self {
        Self {
            r: ((color >> 16) & 0xFF) as u8,
            g: ((color >> 8) & 0xFF) as u8,
            b: (color & 0xFF) as u8,
        }
    }
}

impl From<Rgb> for u32 {
    /// Convert RGB to u32 color value
    ///
    /// e.g. rgb: (1,2,4)
    /// G        R        B
    /// 7      0 7      0 7      0
    /// 00000010 00000001 00000100
    fn from(rgb: Rgb) -> Self {
        ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | rgb.b as u32
    }
}
