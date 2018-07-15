extern crate openni2_sys;
use std::os::raw::{c_int, c_void};
use std::ffi::CStr;
use std::{mem, ptr, slice};
use std::marker::PhantomData;
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
    Pixel,
};
pub use device::{Device, DeviceInfo};
pub use stream::{Stream, StreamListener, StreamReader, Cropping};
pub use frame::Frame;
pub use openni2_sys::{
    OniDepthPixel,
    OniGrayscale16Pixel,
    OniGrayscale8Pixel,
    OniRGB888Pixel,
    OniYUV422DoublePixel,
};

pub fn init(major: c_int, minor: c_int) -> Result<(), Status> {
    match unsafe { oniInitialize(major * 1000 + minor) }.into() {
        Status::Ok => Ok(()),
        error => Err(error),
    }
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

pub fn register_device_callbacks<'a, F1, F2, F3>(on_device_connected: F1, on_device_disconnected: F2, on_device_state_changed: F3) -> Result<DeviceCallbackHandle<'a>, Status>
    where F1: 'a + FnMut(DeviceInfo), F2: 'a + FnMut(DeviceInfo), F3: 'a + FnMut(DeviceInfo, DeviceState) {
    unsafe extern "C" fn on_device_connected_wrapper<F1, F2, F3>(info: *const OniDeviceInfo, cookie: *mut c_void) where F1: FnMut(DeviceInfo), F2: FnMut(DeviceInfo), F3: FnMut(DeviceInfo, DeviceState) {
        let mut closures: Box<ClosureStruct<F1, F2, F3>> = Box::from_raw(cookie as *mut ClosureStruct<F1, F2, F3>);
        let device_info = (*info).into();
        (closures.on_device_connected)(device_info);
        mem::forget(closures);
    }

    unsafe extern "C" fn on_device_disconnected_wrapper<F1, F2, F3>(info: *const OniDeviceInfo, cookie: *mut c_void) where F1: FnMut(DeviceInfo), F2: FnMut(DeviceInfo), F3: FnMut(DeviceInfo, DeviceState) {
        let mut closures: Box<ClosureStruct<F1, F2, F3>> = Box::from_raw(cookie as *mut ClosureStruct<F1, F2, F3>);
        let device_info = (*info).into();
        (closures.on_device_disconnected)(device_info);
        mem::forget(closures);
    }

    unsafe extern "C" fn on_device_state_changed_wrapper<F1, F2, F3>(device_info: *const OniDeviceInfo, device_state: OniDeviceState, cookie: *mut c_void) where F1: FnMut(DeviceInfo), F2: FnMut(DeviceInfo), F3: FnMut(DeviceInfo, DeviceState) {
        let mut closures: Box<ClosureStruct<F1, F2, F3>> = Box::from_raw(cookie as *mut ClosureStruct<F1, F2, F3>);
        let device_info = (*device_info).into();
        let device_state = device_state.into();
        (closures.on_device_state_changed)(device_info, device_state);
        mem::forget(closures);
    }

    let closures = Box::new(ClosureStruct {
        on_device_connected,
        on_device_disconnected,
        on_device_state_changed,
    });

    let mut callbacks = OniDeviceCallbacks {
        deviceConnected: Some(on_device_connected_wrapper::<F1, F2, F3>),
        deviceDisconnected: Some(on_device_disconnected_wrapper::<F1, F2, F3>),
        deviceStateChanged: Some(on_device_state_changed_wrapper::<F1, F2, F3>),
    };

    let mut callbacks_handle: OniCallbackHandle = ptr::null_mut();
    let status = unsafe {
        oniRegisterDeviceCallbacks(&mut callbacks, Box::into_raw(closures) as *mut _, &mut callbacks_handle)
    }.into();

    match status {
        Status::Ok => Ok(DeviceCallbackHandle {
            callbacks_handle,
            _closures_lifetime: PhantomData,
        }),
        _ => Err(status),
    }
}

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
    // FIXME: YUV modes will break the runtime assertions that
    // the expected type param for Frame::pixels() matches the
    // size of the actual array element. OpenNI2 reports that
    // YUV pixels are 2 bytes which they are *but* the struct
    // that holds them is 4 bytes and represents 2 pixels.
    //
    // Must decide if we want to "lie" and return 4 from this
    // function for YUV types (which will not conform to
    // `oniFormatBytesPerPixel` logic) or if we want to change
    // the assertions.
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

#[derive(Debug)]
pub struct DeviceCallbackHandle<'a>{
    callbacks_handle: OniCallbackHandle,
    _closures_lifetime: PhantomData<&'a ()>,
}

impl<'a> DeviceCallbackHandle<'a> {
    pub fn unregister(self) {} // POOF! Bye bye
}

impl<'a> Drop for DeviceCallbackHandle<'a> {
    fn drop(&mut self) {
        unsafe { oniUnregisterDeviceCallbacks(self.callbacks_handle) }
    }
}

struct ClosureStruct<F1, F2, F3>
    where F1: FnMut(DeviceInfo), F2: FnMut(DeviceInfo), F3: FnMut(DeviceInfo, DeviceState) {
    on_device_connected: F1,
    on_device_disconnected: F2,
    on_device_state_changed: F3,
}
