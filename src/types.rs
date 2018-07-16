use std::os::raw::c_int;
use std::fmt;
use openni2_sys::*;
use super::get_extended_error;

#[derive(Debug, Clone)]
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

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_string = match self {
            Status::Ok => "Ok",
            Status::Error(s) => s,
            Status::NotImplemented => "Not implemented",
            Status::NotSupported => "Not supported",
            Status::BadParameter => "Bad parameter",
            Status::OutOfFlow => "Out of flow",
            Status::NoDevice => "No device",
            Status::TimeOut => "Timeout",
        };
        write!(f, "OpenNI2 error: {}", error_string)
    }
}

impl From<c_int> for Status {
    fn from(i: c_int) -> Self {
        Status::from_int(i)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum SensorType {
    IR = ONI_SENSOR_IR,
    COLOR = ONI_SENSOR_COLOR,
    DEPTH = ONI_SENSOR_DEPTH,
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

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum PixelFormat {
    // Depth
    DEPTH_1_MM = ONI_PIXEL_FORMAT_DEPTH_1_MM,
    DEPTH_100_UM = ONI_PIXEL_FORMAT_DEPTH_100_UM,
    SHIFT_9_2 = ONI_PIXEL_FORMAT_SHIFT_9_2,
    SHIFT_9_3 = ONI_PIXEL_FORMAT_SHIFT_9_3,

    // Color
    RGB888 = ONI_PIXEL_FORMAT_RGB888,
    YUV422 = ONI_PIXEL_FORMAT_YUV422,
    GRAY8 = ONI_PIXEL_FORMAT_GRAY8,
    GRAY16 = ONI_PIXEL_FORMAT_GRAY16,
    JPEG = ONI_PIXEL_FORMAT_JPEG,
    YUYV = ONI_PIXEL_FORMAT_YUYV,
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

#[doc(hidden)]
pub fn bytes_per_pixel(format: PixelFormat) -> usize {
    match format {
        PixelFormat::DEPTH_1_MM => 2,
        PixelFormat::DEPTH_100_UM => 2,
        PixelFormat::SHIFT_9_2 => 2,
        PixelFormat::SHIFT_9_3 => 2,
        PixelFormat::RGB888 => 3,
        PixelFormat::YUV422 => 4,
        PixelFormat::GRAY8 => 1,
        PixelFormat::GRAY16 => 2,
        PixelFormat::JPEG => 1,
        PixelFormat::YUYV => 4,
        // _ => unsafe { oniFormatBytesPerPixel(format as i32) as usize },
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum DeviceState {
    OK = ONI_DEVICE_STATE_OK,
    ERROR = ONI_DEVICE_STATE_ERROR,
    NOT_READY = ONI_DEVICE_STATE_NOT_READY,
    EOF = ONI_DEVICE_STATE_EOF,
}

impl DeviceState {
    fn from_int(value: c_int) -> Self {
        match value {
            ONI_DEVICE_STATE_OK => DeviceState::OK,
            ONI_DEVICE_STATE_ERROR => DeviceState::ERROR,
            ONI_DEVICE_STATE_NOT_READY => DeviceState::NOT_READY,
            ONI_DEVICE_STATE_EOF => DeviceState::EOF,
            _ => panic!("Unknown device state {}", value),
        }
    }
}

impl From<c_int> for DeviceState {
    fn from(i: c_int) -> Self {
        DeviceState::from_int(i)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum ImageRegistrationMode {
    OFF = ONI_IMAGE_REGISTRATION_OFF,
    DEPTH_TO_COLOR = ONI_IMAGE_REGISTRATION_DEPTH_TO_COLOR,
}

impl ImageRegistrationMode {
    fn from_int(value: c_int) -> Self {
        match value {
            ONI_IMAGE_REGISTRATION_OFF => ImageRegistrationMode::OFF,
            ONI_IMAGE_REGISTRATION_DEPTH_TO_COLOR => ImageRegistrationMode::DEPTH_TO_COLOR,
            _ => panic!("Unknown image registration mode {}", value),
        }
    }
}

impl From<c_int> for ImageRegistrationMode {
    fn from(i: c_int) -> Self {
        ImageRegistrationMode::from_int(i)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum Timeout {
    NONE = ONI_TIMEOUT_NONE,
    FOREVER = ONI_TIMEOUT_FOREVER,
}

impl Timeout {
    fn from_int(value: c_int) -> Self {
        match value {
            ONI_TIMEOUT_NONE => Timeout::NONE,
            ONI_TIMEOUT_FOREVER => Timeout::FOREVER,
            _ => panic!("Unknown timeout {}", value),
        }
    }
}

impl From<c_int> for Timeout {
    fn from(i: c_int) -> Self {
        Timeout::from_int(i)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct VideoMode {
    pub pixel_format: PixelFormat,
    pub resolution_x: c_int,
    pub resolution_y: c_int,
    pub fps: c_int,
}

impl From<OniVideoMode> for VideoMode {
    fn from(mode: OniVideoMode) -> Self {
        VideoMode {
            pixel_format: mode.pixelFormat.into(),
            resolution_x: mode.resolutionX,
            resolution_y: mode.resolutionY,
            fps: mode.fps,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Version {
    pub major: c_int,
    pub minor: c_int,
    pub maintenance: c_int,
    pub build: c_int,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.major, self.minor, self.maintenance, self.build)
    }
}

#[derive(Debug)]
pub struct SensorInfo {
    pub sensor_type: SensorType,
    pub video_modes: Vec<VideoMode>,
}

macro_rules! isPixel {
    ($($in:ty),+) => (
        pub trait Pixel: Copy + fmt::Debug {}
        $(impl Pixel for $in {})+
    )
}
isPixel!(OniDepthPixel, /*OniGrayscale16Pixel,*/ OniGrayscale8Pixel, OniRGB888Pixel, OniYUV422DoublePixel);

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum LogLevel {
    None = 10,
    Verbose = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}
