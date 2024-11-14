use crate::consts;
use error_stack::ResultExt;
use esp_idf_svc::hal::uart::{AsyncUartDriver, UartDriver};
use esp_idf_svc::io::asynch::{Read, Write};
use log::{info, warn};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error("read error")]
    ReadError,
    #[error("write error")]
    WriteError,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Request {
    InitTest(u8),
    AnalogWrite(AnalogWritePort, u16),
    AnalogRead(AnalogReadPort),
    DigitalWrite(DigitalPort, u8),
    DigitalRead(DigitalPort),
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum AnalogWritePort {
    Port1,
    Port2,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DigitalPort {
    Port1,
    Port2,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum AnalogReadPort {
    Port1 = 0,
    Port2 = 1,
    Port3 = 2,
    Port4 = 3,
    Port5 = 4,
    Port6 = 5,
    Port7 = 6,
    Port8 = 7,
}

pub async fn read_request<'d>(
    uart: &mut AsyncUartDriver<'d, UartDriver<'d>>,
) -> error_stack::Result<Option<Request>, CommunicationError> {
    let mut instructions_buffer = [0u8; 1];
    uart.read_exact(&mut instructions_buffer)
        .await
        .change_context(CommunicationError::ReadError)?;
    let [instruction] = instructions_buffer;
    info!("Got Instruction: {instruction:#x}");
    match instruction {
        consts::RQ_TEST => {
            let mut buffer = [0u8; 1];
            uart.read_exact(&mut buffer)
                .await
                .change_context(CommunicationError::ReadError)?;
            let [test] = buffer;
            Ok(Some(Request::InitTest(test)))
        }
        consts::RQ_ANALOG_WRITE_0 | consts::RQ_ANALOG_WRITE_1 => {
            let port = if instruction == consts::RQ_ANALOG_WRITE_0 {
                AnalogWritePort::Port1
            } else {
                AnalogWritePort::Port2
            };
            let mut buffer = [0u8; 2];
            uart.read_exact(&mut buffer)
                .await
                .change_context(CommunicationError::ReadError)?;
            let [value_low, value_high] = buffer;
            let value = (value_high as u16) << 8 | value_low as u16;
            Ok(Some(Request::AnalogWrite(port, value)))
        }
        consts::RQ_ANALOG_READ => {
            let mut buffer = [0u8; 1];
            uart.read_exact(&mut buffer)
                .await
                .change_context(CommunicationError::ReadError)?;
            let [port] = buffer;

            let port = match port {
                0 => AnalogReadPort::Port1,
                1 => AnalogReadPort::Port2,
                2 => AnalogReadPort::Port3,
                3 => AnalogReadPort::Port4,
                4 => AnalogReadPort::Port5,
                5 => AnalogReadPort::Port6,
                6 => AnalogReadPort::Port7,
                7 => AnalogReadPort::Port8,
                _ => return Ok(None),
            };
            Ok(Some(Request::AnalogRead(port)))
        }

        consts::RQ_DIGITAL_WRITE_0 | consts::RQ_DIGITAL_WRITE_1 => {
            let port = if instruction == consts::RQ_DIGITAL_WRITE_0 {
                DigitalPort::Port1
            } else {
                DigitalPort::Port2
            };
            let mut buffer = [0u8; 1];
            uart.read_exact(&mut buffer)
                .await
                .change_context(CommunicationError::ReadError)?;
            let [value] = buffer;
            Ok(Some(Request::DigitalWrite(port, value)))
        }
        consts::RQ_DIGITAL_READ_0 | consts::RQ_DIGITAL_READ_1 => {
            let port = if instruction == consts::RQ_DIGITAL_READ_0 {
                DigitalPort::Port1
            } else {
                DigitalPort::Port2
            };
            Ok(Some(Request::DigitalRead(port)))
        }

        instruction => {
            warn!("Received unknown instruction: {instruction:#x}");
            Ok(None)
        }
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Response {
    Ok,
    Error,
    TestEcho(u8),
    AnalogValue(u16),
    DigitalValue(u8),
}
pub async fn write_response<'d>(
    uart: &mut AsyncUartDriver<'d, UartDriver<'d>>,
    response: Response,
) -> error_stack::Result<(), CommunicationError> {
    match response {
        Response::Ok => {
            uart.write_all(&[consts::MSG_OK])
                .await
                .change_context(CommunicationError::WriteError)?;
        }
        Response::TestEcho(value) => {
            uart.write_all(&[consts::MSG_OK, value])
                .await
                .change_context(CommunicationError::WriteError)?;
        }
        Response::AnalogValue(value) => {
            uart.write_all(&value.to_le_bytes())
                .await
                .change_context(CommunicationError::WriteError)?;
        }
        Response::DigitalValue(value) => {
            uart.write_all(&value.to_le_bytes())
                .await
                .change_context(CommunicationError::WriteError)?;
        }
        Response::Error => {
            uart.write_all(&[consts::MSG_ERROR])
                .await
                .change_context(CommunicationError::WriteError)?;
        }
    }
    uart.flush()
        .await
        .change_context(CommunicationError::WriteError)?;
    Ok(())
}
