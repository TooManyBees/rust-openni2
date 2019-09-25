use std::os::raw::c_int;
use std::fmt;
use openni2_sys::*;
use super::get_extended_error;

/// Error state for external OpenNI2 C functions
#[derive(Debug, Clone)]
pub enum Status {
    /// Success
    Ok,
    /// Some error with a message set by OpenNI2
    Error(String),
    NotImplemented,
    /// Attempted to set a property not supported by a Stream
    NotSupported,
    /// Attempted to set a property with an invalid parameter
    BadParameter,
    /// Stream was not in a required state, like running/stopped
    OutOfFlow,
    NoDevice,
    /// A timeout expired before an operation succeeded
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

/// One of the supported sensor types of a device
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

pub trait Pixel: fmt::Debug {
    type Format;
    const BYTES_PER_PIXEL: usize;
    const ONI_PIXEL_FORMAT: i32;
}

macro_rules! pixel {
    ($in:ident, $px:ty, $enum:expr, $bpp:literal) => {
        #[derive(Debug)]
        pub struct $in(pub $px);
        impl Pixel for $in {
            type Format = $px;
            const BYTES_PER_PIXEL: usize = $bpp;
            const ONI_PIXEL_FORMAT: i32 = $enum;
        }
    }
}

pixel!(DepthPixel1MM, OniDepthPixel, ONI_PIXEL_FORMAT_DEPTH_1_MM, 2);
pixel!(DepthPixel100UM, OniDepthPixel, ONI_PIXEL_FORMAT_DEPTH_100_UM, 2);
pixel!(DepthPixelShift92, OniDepthPixel, ONI_PIXEL_FORMAT_SHIFT_9_2, 2);
pixel!(DepthPixelShift93, OniDepthPixel, ONI_PIXEL_FORMAT_SHIFT_9_3, 2);
pixel!(ColorPixelRGB888, OniRGB888Pixel, ONI_PIXEL_FORMAT_RGB888, 3);
pixel!(ColorPixelYUV442, OniYUV422DoublePixel, ONI_PIXEL_FORMAT_YUV422, 4);
pixel!(ColorPixelGray8, OniGrayscale8Pixel, ONI_PIXEL_FORMAT_GRAY8, 1);
pixel!(ColorPixelGray16, OniGrayscale16Pixel, ONI_PIXEL_FORMAT_GRAY16, 2);
pixel!(ColorPixelJpeg, u8, ONI_PIXEL_FORMAT_JPEG, 1);
pixel!(ColorPixelYUYV, OniYUV422DoublePixel, ONI_PIXEL_FORMAT_YUYV, 4);

pub(crate) unsafe fn bytes_per_pixel(format: i32) -> usize {
    match format {
        ONI_PIXEL_FORMAT_DEPTH_1_MM => 2,
        ONI_PIXEL_FORMAT_DEPTH_100_UM => 2,
        ONI_PIXEL_FORMAT_SHIFT_9_2 => 2,
        ONI_PIXEL_FORMAT_SHIFT_9_3 => 2,
        ONI_PIXEL_FORMAT_RGB888 => 3,
        ONI_PIXEL_FORMAT_YUV422 => 4,
        ONI_PIXEL_FORMAT_GRAY8 => 1,
        ONI_PIXEL_FORMAT_GRAY16 => 2,
        ONI_PIXEL_FORMAT_JPEG => 1,
        ONI_PIXEL_FORMAT_YUYV => 4,
         _ => oniFormatBytesPerPixel(format) as usize,
    }
}

/// Current state of a device.
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

/// Mode for automatically resizing/translating stream outputs
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum ImageRegistrationMode {
    /// Don't align any streams
    OFF = ONI_IMAGE_REGISTRATION_OFF,
    /// Resize and reposition the depth stream to match the color stream
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

/// Dimensions and framerate of a stream.
///
/// Returned as current video mode of a stream, or passed as
/// the desired video mode when updating a stream.
#[derive(Debug, Copy, Clone)]
pub struct VideoMode {
    pub resolution_x: c_int,
    pub resolution_y: c_int,
    pub fps: c_int,
}

impl From<OniVideoMode> for VideoMode {
    fn from(mode: OniVideoMode) -> Self {
        VideoMode {
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

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum LogLevel {
    None = 10,
    Verbose = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}
