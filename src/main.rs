mod com;
mod consts;
mod neopixel;
mod pins;

use crate::com::{DigitalPort, Request, Response};
use crate::neopixel::{Neopixel, Rgb};
use crate::pins::{
    ASidePinDrivers, BSidePinDrivers, PinDriversAnalogA, PinDriversDigitalA, PinDriversDigitalB,
    PinsA, PinsB,
};
use error_stack::{Result, ResultExt};
use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::ADC1;
use esp_idf_svc::hal::gpio::{Gpio16, Gpio17};
use esp_idf_svc::hal::prelude::Peripherals;
#[cfg(feature = "rt-embassy")]
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::hal::uart::config::{DataBits, FlowControl};
use esp_idf_svc::hal::uart::{AsyncUartDriver, UartConfig, UartDriver};
use esp_idf_svc::hal::units::Hertz;
use log::{error, warn};
use thiserror::Error;
#[cfg(feature = "log")]
use tracing::{debug, info};

#[cfg(not(any(feature = "rt-tokio", feature = "rt-embassy")))]
compile_error!(
    "No async runtime selected. Please select one of the following features: rt-tokio, rt-embassy"
);
#[cfg(all(feature = "rt-tokio", feature = "rt-embassy"))]
compile_error!("Multiple async runtimes selected. Please select only one of the following features: rt-tokio, rt-embassy");

#[derive(Debug, Error)]
pub enum B32Error {
    #[error("create runtime")]
    CreateRuntime,
    #[error("esp32 error")]
    Esp32Error,
    #[error("communication error")]
    CommunicationError,
}

fn main() -> error_stack::Result<(), B32Error> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    #[cfg(feature = "rt-tokio")]
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .thread_stack_size(consts::STACK_SIZE)
        .build()
        .change_context(B32Error::CreateRuntime)?;

    #[cfg(feature = "log")]
    info!("Initializing...");
    let peripherals = Peripherals::take().change_context(B32Error::Esp32Error)?;
    let mut led = Neopixel::new(peripherals.pins.gpio8, peripherals.rmt.channel0);
    led.set_color(Rgb::new(64, 64, 0))
        .change_context(B32Error::Esp32Error)?;

    //Analog pins
    let adc = AdcDriver::new(peripherals.adc1).change_context(B32Error::Esp32Error)?;
    let adc_channel_config = AdcChannelConfig::new();

    let pins1 = PinsA {
        a0_ad: peripherals.pins.gpio2,
        a1_ad: peripherals.pins.gpio3,
        a2_ad: peripherals.pins.gpio4,
        a3_ad: peripherals.pins.gpio5,
        a4_ad: peripherals.pins.gpio0,
        a5_ad: peripherals.pins.gpio1,
        a7_d: peripherals.pins.gpio14,
    };

    let pins2 = PinsB {
        b0_d: peripherals.pins.gpio23,
        b1_d: peripherals.pins.gpio22,
        b2_d: peripherals.pins.gpio21,
        b3_d: peripherals.pins.gpio20,
        b4_d: peripherals.pins.gpio19,
        b5_d: peripherals.pins.gpio18,
        b6_d: peripherals.pins.gpio15,
        b7_d: peripherals.pins.gpio9,
    };

    #[cfg(feature = "log")]
    info!("Opening serial port...");
    let usb_serial_config = UartConfig::new()
        .data_bits(DataBits::DataBits8)
        .parity_none()
        .baudrate(Hertz(consts::BAUD))
        .flow_control(FlowControl::None);

    let mut usb_serial = AsyncUartDriver::new(
        peripherals.uart1,
        peripherals.pins.gpio16,
        peripherals.pins.gpio17,
        Option::<Gpio16>::None,
        Option::<Gpio17>::None,
        &usb_serial_config,
    )
    .change_context(B32Error::Esp32Error)?;
    #[cfg(feature = "log")]
    info!("Serial port opened");
    let runtime_fn = app_main(
        &mut usb_serial,
        &mut led,
        pins1,
        pins2,
        adc,
        &adc_channel_config,
    );
    #[cfg(feature = "rt-tokio")]
    let result = runtime.block_on(runtime_fn);
    #[cfg(feature = "rt-embassy")]
    let result = block_on(runtime_fn);

    if let Result::Err(err) = &result {
        #[cfg(feature = "log")]
        error!("Main loop failed: {err:#?}");
    } else {
        #[cfg(feature = "log")]
        warn!("Main loop returned without error, that's weird");
    }

    led.set_color(Rgb::new(64, 0, 0))
        .change_context(B32Error::Esp32Error)?;
    result
}

async fn app_main<'d>(
    usb_serial: &mut AsyncUartDriver<'d, UartDriver<'d>>,
    led: &mut Neopixel,
    mut pinsa: PinsA,
    mut pinsb: PinsB,
    adc: AdcDriver<'d, ADC1>,
    adc_channel_config: &AdcChannelConfig,
) -> error_stack::Result<(), B32Error> {
    let mut a_side = ASidePinDrivers::None;
    let mut b_side = BSidePinDrivers::None;

    loop {
        #[cfg(feature = "log")]
        info!("Waiting for instructions...");
        led.set_color(Rgb::new(0, 64, 0))
            .change_context(B32Error::Esp32Error)?;
        let request = com::read_request(usb_serial).await;
        led.set_color(Rgb::new(0, 0, 128))
            .change_context(B32Error::Esp32Error)?;
        match request {
            Ok(request) => {
                #[cfg(feature = "log")]
                debug!("Got request: {:?}", request);
                match request {
                    Some(Request::InitTest(value)) => {
                        com::write_response(usb_serial, Response::TestEcho(value))
                            .await
                            .change_context(B32Error::Esp32Error)?;
                    }
                    Some(Request::AnalogRead(port)) => {
                        let (output, analog_read) =
                            if let ASidePinDrivers::Analog(mut analog_read) = a_side {
                                (
                                    analog_read
                                        .analog_read(&adc, port)
                                        .change_context(B32Error::Esp32Error)?,
                                    ASidePinDrivers::Analog(analog_read),
                                )
                            } else {
                                drop(a_side);
                                let mut analog_read = PinDriversAnalogA {
                                    a0_ad: AdcChannelDriver::new(
                                        &adc,
                                        &mut pinsa.a0_ad,
                                        adc_channel_config,
                                    )
                                    .change_context(B32Error::Esp32Error)?,
                                    a1_ad: AdcChannelDriver::new(
                                        &adc,
                                        &mut pinsa.a1_ad,
                                        adc_channel_config,
                                    )
                                    .change_context(B32Error::Esp32Error)?,
                                    a2_ad: AdcChannelDriver::new(
                                        &adc,
                                        &mut pinsa.a2_ad,
                                        adc_channel_config,
                                    )
                                    .change_context(B32Error::Esp32Error)?,
                                    a3_ad: AdcChannelDriver::new(
                                        &adc,
                                        &mut pinsa.a3_ad,
                                        adc_channel_config,
                                    )
                                    .change_context(B32Error::Esp32Error)?,
                                    a4_ad: AdcChannelDriver::new(
                                        &adc,
                                        &mut pinsa.a4_ad,
                                        adc_channel_config,
                                    )
                                    .change_context(B32Error::Esp32Error)?,
                                    a5_ad: AdcChannelDriver::new(
                                        &adc,
                                        &mut pinsa.a5_ad,
                                        adc_channel_config,
                                    )
                                    .change_context(B32Error::Esp32Error)?,
                                };
                                (
                                    analog_read
                                        .analog_read(&adc, port)
                                        .change_context(B32Error::Esp32Error)?,
                                    ASidePinDrivers::Analog(analog_read),
                                )
                            };
                        a_side = analog_read;
                        com::write_response(usb_serial, Response::AnalogValue(output))
                            .await
                            .change_context(B32Error::Esp32Error)?;
                    }
                    Some(Request::DigitalRead(port)) => {
                        let output = match port {
                            DigitalPort::Port1 => {
                                let (output, digital_read) =
                                    if let ASidePinDrivers::Digital(mut digital_read) = a_side {
                                        (
                                            digital_read
                                                .digital_read()
                                                .change_context(B32Error::Esp32Error)?,
                                            digital_read,
                                        )
                                    } else {
                                        drop(a_side);
                                        let mut digital_read = PinDriversDigitalA::new(&mut pinsa)
                                            .change_context(B32Error::Esp32Error)?;
                                        (
                                            digital_read
                                                .digital_read()
                                                .change_context(B32Error::Esp32Error)?,
                                            digital_read,
                                        )
                                    };

                                a_side = ASidePinDrivers::Digital(digital_read);
                                output
                            }
                            DigitalPort::Port2 => {
                                let (output, digital_read) =
                                    if let BSidePinDrivers::Digital(mut digital_read) = b_side {
                                        (
                                            digital_read
                                                .digital_read()
                                                .change_context(B32Error::Esp32Error)?,
                                            digital_read,
                                        )
                                    } else {
                                        drop(b_side);
                                        let mut digital_read = PinDriversDigitalB::new(&mut pinsb)
                                            .change_context(B32Error::Esp32Error)?;
                                        (
                                            digital_read
                                                .digital_read()
                                                .change_context(B32Error::Esp32Error)?,
                                            digital_read,
                                        )
                                    };

                                b_side = BSidePinDrivers::Digital(digital_read);
                                output
                            }
                        };
                        com::write_response(usb_serial, Response::DigitalValue(output))
                            .await
                            .change_context(B32Error::CommunicationError)?;
                    }
                    Some(Request::DigitalWrite(port, value)) => {
                        match port {
                            DigitalPort::Port1 => {
                                a_side = if let ASidePinDrivers::Digital(mut digital_write) = a_side
                                {
                                    digital_write
                                        .digital_write(value)
                                        .change_context(B32Error::Esp32Error)?;
                                    ASidePinDrivers::Digital(digital_write)
                                } else {
                                    drop(a_side);
                                    let mut digital_write = PinDriversDigitalA::new(&mut pinsa)
                                        .change_context(B32Error::Esp32Error)?;
                                    digital_write
                                        .digital_write(value)
                                        .change_context(B32Error::Esp32Error)?;
                                    ASidePinDrivers::Digital(digital_write)
                                }
                            }
                            DigitalPort::Port2 => {
                                b_side = if let BSidePinDrivers::Digital(mut digital_write) = b_side
                                {
                                    digital_write
                                        .digital_write(value)
                                        .change_context(B32Error::Esp32Error)?;
                                    BSidePinDrivers::Digital(digital_write)
                                } else {
                                    drop(b_side);
                                    let mut digital_write = PinDriversDigitalB::new(&mut pinsb)
                                        .change_context(B32Error::Esp32Error)?;
                                    digital_write
                                        .digital_write(value)
                                        .change_context(B32Error::Esp32Error)?;
                                    BSidePinDrivers::Digital(digital_write)
                                }
                            }
                        }

                        com::write_response(usb_serial, Response::Ok)
                            .await
                            .change_context(B32Error::Esp32Error)?;
                    }
                    _ => {
                        warn!(
                            "Unknown or malformed request received therefore responding with error"
                        );
                        com::write_response(usb_serial, Response::Error)
                            .await
                            .change_context(B32Error::Esp32Error)?;
                    }
                }
            }
            Err(err) => {
                com::write_response(usb_serial, Response::Error)
                    .await
                    .change_context(B32Error::Esp32Error)?;
                return Err(err.change_context(B32Error::CommunicationError));
            }
        }
    }
}
