use std::os::raw::c_int;
use openni2_sys::*;
use super::get_extended_error;

#[derive(Debug)]
pub enum Status {
    Ok,
    Error(String),
    NotImplemented,
    NotSupported,
    BadParameter,
    OutOfFlow,
    NoDevice,
    TimeOut,
}

impl Status {
    fn from_int(value: c_int) -> Self {
        match value {
            ONI_STATUS_OK => Status::Ok,
            ONI_STATUS_ERROR => Status::Error(get_extended_error()),
            ONI_STATUS_NOT_IMPLEMENTED => Status::NotImplemented,
            ONI_STATUS_NOT_SUPPORTED => Status::NotSupported,
            ONI_STATUS_BAD_PARAMETER => Status::BadParameter,
            ONI_STATUS_OUT_OF_FLOW => Status::OutOfFlow,
            ONI_STATUS_NO_DEVICE => Status::NoDevice,
            ONI_STATUS_TIME_OUT => Status::TimeOut,
            _ => Status::Error(format!("Unknown status code {}", value))
        }
    }
}

impl From<c_int> for Status {
    fn from(i: c_int) -> Self {
        Status::from_int(i)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum PixelFormat {
    // Depth
    DEPTH_1_MM = ONI_PIXEL_FORMAT_DEPTH_1_MM as isize,
    DEPTH_100_UM = ONI_PIXEL_FORMAT_DEPTH_100_UM as isize,
    SHIFT_9_2 = ONI_PIXEL_FORMAT_SHIFT_9_2 as isize,
    SHIFT_9_3 = ONI_PIXEL_FORMAT_SHIFT_9_3 as isize,

    // Color
    RGB888 = ONI_PIXEL_FORMAT_RGB888 as isize,
    YUV422 = ONI_PIXEL_FORMAT_YUV422 as isize,
    GRAY8 = ONI_PIXEL_FORMAT_GRAY8 as isize,
    GRAY16 = ONI_PIXEL_FORMAT_GRAY16 as isize,
    JPEG = ONI_PIXEL_FORMAT_JPEG as isize,
    YUYV = ONI_PIXEL_FORMAT_YUYV as isize,
}

impl PixelFormat {
    fn from_int(value: c_int) -> Self {
        match value {
            ONI_PIXEL_FORMAT_DEPTH_1_MM => PixelFormat::DEPTH_1_MM,
            ONI_PIXEL_FORMAT_DEPTH_100_UM => PixelFormat::DEPTH_100_UM,
            ONI_PIXEL_FORMAT_SHIFT_9_2 => PixelFormat::SHIFT_9_2,
            ONI_PIXEL_FORMAT_SHIFT_9_3 => PixelFormat::SHIFT_9_3,
            ONI_PIXEL_FORMAT_RGB888 => PixelFormat::RGB888,
            ONI_PIXEL_FORMAT_YUV422 => PixelFormat::YUV422,
            ONI_PIXEL_FORMAT_GRAY8 => PixelFormat::GRAY8,
            ONI_PIXEL_FORMAT_GRAY16 => PixelFormat::GRAY16,
            ONI_PIXEL_FORMAT_JPEG => PixelFormat::JPEG,
            ONI_PIXEL_FORMAT_YUYV => PixelFormat::YUYV,
            _ => panic!("Unknown pixel format {}", value),
        }
    }
}

impl From<c_int> for PixelFormat {
    fn from(i: c_int) -> Self {
        PixelFormat::from_int(i)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum SensorType {
    IR = ONI_SENSOR_IR as isize,
    COLOR = ONI_SENSOR_COLOR as isize,
    DEPTH = ONI_SENSOR_DEPTH as isize,
}

impl SensorType {
    fn from_int(value: c_int) -> Self {
        match value {
            ONI_SENSOR_IR => SensorType::IR,
            ONI_SENSOR_COLOR => SensorType::COLOR,
            ONI_SENSOR_DEPTH => SensorType::DEPTH,
            _ => panic!("Unknown sensor type {}", value),
        }
    }
}

impl From<c_int> for SensorType {
    fn from(i: c_int) -> Self {
        SensorType::from_int(i)
    }
}
