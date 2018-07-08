extern crate openni2_sys;
use std::os::raw::{c_int};
use std::ffi::CStr;
use std::{ptr, slice};
use openni2_sys::*;

mod device;
mod stream;
mod frame;
mod types;

pub use types::{
    Status,
    SensorType,
    PixelFormat,
    DeviceState,
    ImageRegistrationMode,
    Timeout,
    VideoMode,
};
pub use device::{Device, DeviceInfo};

pub fn init(major: c_int, minor: c_int) -> Status {
    unsafe { oniInitialize(major * 1000 + minor) }.into()
}

pub fn shutdown() {
    unsafe { oniShutdown() };
}

// FIXME: returning a private OniVersion type
pub fn get_version() -> OniVersion {
    unsafe { oniGetVersion() }
}

fn get_extended_error() -> String {
    let string = unsafe {
        let err_ptr = oniGetExtendedError();
        CStr::from_ptr(err_ptr)
    };
    match string.to_str() {
        Ok(s) => s.trim().to_string(),
        Err(_) => "Unknown error".to_string(),
    }
}

pub fn get_device_list() -> Vec<DeviceInfo> {
    let mut pointer = ptr::null_mut();
    let mut count = ONI_MAX_SENSORS as c_int;
    let devices: &[OniDeviceInfo] = unsafe {
        oniGetDeviceList(&mut pointer, &mut count);
        assert!(!pointer.is_null());
        slice::from_raw_parts(pointer, count as usize)
    };
    let mapped = devices.iter().map(|&info| info.into()).collect();
    unsafe { oniReleaseDeviceList(pointer); }
    mapped
}

// pub fn register_device_callbacks() -> Status {

// }

// pub fn unregister_device_callbacks() {

// }

// pub fn wait_for_any_stream() -> Status {

// }

pub fn set_console_log(state: bool) -> Status {
    let return_value = unsafe {
        if state {
            oniSetLogConsoleOutput(1)
        } else {
            oniSetLogConsoleOutput(2)
        }
    };
    unsafe { oniSetLogMinSeverity(0) };
    return_value.into()
}

pub fn bytes_per_pixel(format: PixelFormat) -> usize {
    match format {
        PixelFormat::DEPTH_1_MM => 2,
        PixelFormat::DEPTH_100_UM => 2,
        PixelFormat::SHIFT_9_2 => 2,
        PixelFormat::SHIFT_9_3 => 2,
        PixelFormat::RGB888 => 3,
        PixelFormat::YUV422 => 2,
        PixelFormat::GRAY8 => 1,
        PixelFormat::GRAY16 => 2,
        PixelFormat::JPEG => 1,
        PixelFormat::YUYV => 2,
        // _ => unsafe { oniFormatBytesPerPixel(format as i32) as usize },
    }
}
