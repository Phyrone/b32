use crate::com::AnalogReadPort;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::ADC1;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::sys::EspError;

pub struct PinsA {
    // Side A
    pub a0_ad: Gpio2,
    pub a1_ad: Gpio3,
    pub a2_ad: Gpio4,
    pub a3_ad: Gpio5,
    pub a4_ad: Gpio0,
    pub a5_ad: Gpio1,
    pub a7_d: Gpio14,
}

pub struct PinDriversDigitalA<'p> {
    //Side A
    //TODO: add support for ADC channels
    pub a0_ad: PinDriver<'p, Gpio2, InputOutput>,
    pub a1_ad: PinDriver<'p, Gpio3, InputOutput>,
    pub a2_ad: PinDriver<'p, Gpio4, InputOutput>,
    pub a3_ad: PinDriver<'p, Gpio5, InputOutput>,
    pub a4_ad: PinDriver<'p, Gpio0, InputOutput>,
    pub a5_ad: PinDriver<'p, Gpio1, InputOutput>,
    pub a7_d: PinDriver<'p, Gpio14, InputOutput>,
}

impl<'p> PinDriversDigitalA<'p> {
    
    pub fn new(pins: &'p mut PinsA) -> Result<Self, EspError> {
        let a0_ad = PinDriver::input_output(&mut pins.a0_ad)?;
        let a1_ad = PinDriver::input_output(&mut pins.a1_ad)?;
        let a2_ad = PinDriver::input_output(&mut pins.a2_ad)?;
        let a3_ad = PinDriver::input_output(&mut pins.a3_ad)?;
        let a4_ad = PinDriver::input_output(&mut pins.a4_ad)?;
        let a5_ad = PinDriver::input_output(&mut pins.a5_ad)?;
        let a7_d = PinDriver::input_output(&mut pins.a7_d)?;

        Ok(Self {
            a0_ad,
            a1_ad,
            a2_ad,
            a3_ad,
            a4_ad,
            a5_ad,
            a7_d,
        })
    }
    
    pub fn digital_read(&mut self) -> Result<u8, EspError> {
        const POS_MASKS: [u8; 8] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];
        let bits: [bool; 7] = [
            self.a0_ad.is_high(),
            self.a1_ad.is_high(),
            self.a2_ad.is_high(),
            self.a3_ad.is_high(),
            self.a4_ad.is_high(),
            self.a5_ad.is_high(),
            self.a7_d.is_high(),
        ];

        let mut result = 0;
        for (i, entry) in bits.iter().enumerate() {
            if *entry {
                result |= POS_MASKS[i];
            }
        }
        Ok(result)
    }
    pub fn digital_write(&mut self, value: u8) -> Result<(), EspError> {
        let mut levels = [Level::Low; 8];
        for (i, entry) in levels.iter_mut().enumerate() {
            *entry = if (value & (1 << i)) != 0 {
                Level::High
            } else {
                Level::Low
            };
        }
        self.a0_ad.set_level(levels[0])?;
        self.a1_ad.set_level(levels[1])?;
        self.a2_ad.set_level(levels[2])?;
        self.a3_ad.set_level(levels[3])?;
        self.a4_ad.set_level(levels[4])?;
        self.a5_ad.set_level(levels[5])?;
        self.a7_d.set_level(levels[6])?;
        Ok(())
    }
}

pub struct PinDriversAnalogA<'p, 'd> {
    //Side A
    pub a0_ad: AdcChannelDriver<'d, Gpio2, &'p AdcDriver<'d, ADC1>>,
    pub a1_ad: AdcChannelDriver<'d, Gpio3, &'p AdcDriver<'d, ADC1>>,
    pub a2_ad: AdcChannelDriver<'d, Gpio4, &'p AdcDriver<'d, ADC1>>,
    pub a3_ad: AdcChannelDriver<'d, Gpio5, &'p AdcDriver<'d, ADC1>>,
    pub a4_ad: AdcChannelDriver<'d, Gpio0, &'p AdcDriver<'d, ADC1>>,
    pub a5_ad: AdcChannelDriver<'d, Gpio1, &'p AdcDriver<'d, ADC1>>,

}

impl<'p, 'd> PinDriversAnalogA<'p, 'd> {
    
    pub fn analog_read(&mut self, adc: &'p AdcDriver<'d, ADC1>, port: AnalogReadPort) -> Result<u16, EspError> {
        match port {
            AnalogReadPort::Port1 => adc.read(&mut self.a0_ad),
            AnalogReadPort::Port2 => adc.read(&mut self.a1_ad),
            AnalogReadPort::Port3 => adc.read(&mut self.a2_ad),
            AnalogReadPort::Port4 => adc.read(&mut self.a3_ad),
            AnalogReadPort::Port5 => adc.read(&mut self.a4_ad),
            AnalogReadPort::Port6 => adc.read(&mut self.a5_ad),
            AnalogReadPort::Port7 | AnalogReadPort::Port8 => Ok(0), //ADC does not have enough channels
        }
    }
}

pub enum ASidePinDrivers<'p, 'd> {
    None,
    Digital(PinDriversDigitalA<'p>),
    Analog(PinDriversAnalogA<'p, 'd>),
}


pub struct PinsB {
    // Side B
    pub b0_d: Gpio23,
    pub b1_d: Gpio22,
    pub b2_d: Gpio21,
    pub b3_d: Gpio20,
    pub b4_d: Gpio19,
    pub b5_d: Gpio18,
    pub b6_d: Gpio15,
    pub b7_d: Gpio9,
}

pub struct PinDriversDigitalB<'p> {
    //Side B
    pub b0_d: PinDriver<'p, Gpio23, InputOutput>,
    pub b1_d: PinDriver<'p, Gpio22, InputOutput>,
    pub b2_d: PinDriver<'p, Gpio21, InputOutput>,
    pub b3_d: PinDriver<'p, Gpio20, InputOutput>,
    pub b4_d: PinDriver<'p, Gpio19, InputOutput>,
    pub b5_d: PinDriver<'p, Gpio18, InputOutput>,
    pub b6_d: PinDriver<'p, Gpio15, InputOutput>,
    pub b7_d: PinDriver<'p, Gpio9, InputOutput>,
}

impl<'p> PinDriversDigitalB<'p> {
    
    pub fn new(pins: &'p mut PinsB) -> Result<Self, EspError> {
        let b0_d = PinDriver::input_output(&mut pins.b0_d)?;
        let b1_d = PinDriver::input_output(&mut pins.b1_d)?;
        let b2_d = PinDriver::input_output(&mut pins.b2_d)?;
        let b3_d = PinDriver::input_output(&mut pins.b3_d)?;
        let b4_d = PinDriver::input_output(&mut pins.b4_d)?;
        let b5_d = PinDriver::input_output(&mut pins.b5_d)?;
        let b6_d = PinDriver::input_output(&mut pins.b6_d)?;
        let b7_d = PinDriver::input_output(&mut pins.b7_d)?;

        Ok(Self {
            b0_d,
            b1_d,
            b2_d,
            b3_d,
            b4_d,
            b5_d,
            b6_d,
            b7_d,
        })
    }
    
    pub fn digital_read(&mut self) -> Result<u8, EspError> {
        const POS_MASKS: [u8; 8] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];
        let bits: [bool; 8] = [
            self.b0_d.is_high(),
            self.b1_d.is_high(),
            self.b2_d.is_high(),
            self.b3_d.is_high(),
            self.b4_d.is_high(),
            self.b5_d.is_high(),
            self.b6_d.is_high(),
            self.b7_d.is_high(),
        ];

        let mut result = 0;
        for (i, entry) in bits.iter().enumerate() {
            if *entry {
                result |= POS_MASKS[i];
            }
        }
        Ok(result)
    }

    pub fn digital_write(&mut self, value: u8) -> Result<(), EspError> {
        let mut levels = [Level::Low; 8];
        for (i, entry) in levels.iter_mut().enumerate() {
            *entry = if (value & (1 << i)) != 0 {
                Level::High
            } else {
                Level::Low
            };
        }
        self.b0_d.set_level(levels[0])?;
        self.b1_d.set_level(levels[1])?;
        self.b2_d.set_level(levels[2])?;
        self.b3_d.set_level(levels[3])?;
        self.b4_d.set_level(levels[4])?;
        self.b5_d.set_level(levels[5])?;
        self.b6_d.set_level(levels[6])?;
        self.b7_d.set_level(levels[7])?;
        Ok(())
    }
}

pub enum BSidePinDrivers<'p> {
    None,
    Digital(PinDriversDigitalB<'p>),
}
