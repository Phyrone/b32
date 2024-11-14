//Serial port settings
pub const BAUD: u32 = 57600;

pub const MSG_OK: u8 = 0xFF;
pub const MSG_ERROR: u8 = 0xFE;
pub const MAX_DATA_SIZE: u8 = 64;

//Requests
pub const RQ_DISCARD: u8 = 0;
pub const RQ_TEST: u8 = 1;
pub const RQ_INFO: u8 = 2;
pub const RQ_INT_TEST: u8 = 3;
pub const RQ_SELF_TEST: u8 = 4;
pub const RQ_DIGITAL_WRITE_0: u8 = 5;
pub const RQ_DIGITAL_WRITE_1: u8 = 6;
pub const RQ_DIGITAL_READ_0: u8 = 7;
pub const RQ_DIGITAL_READ_1: u8 = 8;
pub const RQ_READ_DIP_SWITCH: u8 = 9;
pub const RQ_ANALOG_WRITE_0: u8 = 10;
pub const RQ_ANALOG_WRITE_1: u8 = 11;
pub const RQ_ANALOG_READ: u8 = 12;
pub const RQ_ADC_DAC_STROKE: u8 = 13;
pub const RQ_PWM_SET_FREQ: u8 = 14;
pub const RQ_PWM_SET_VALUE: u8 = 15;

pub const STACK_SIZE: usize = 1024 * 64;
