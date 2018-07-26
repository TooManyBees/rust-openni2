//! # Rust wrapper for OpenNI2
//!
//! OpenNI2 is a library for handling video streams as part of a natural
//! interaction user interface. Devices such as the Xbox Kinect or Asus
//! Xtion have drivers recognized by OpenNI2.
//!
//! The flow of a program starts with initializing OpenNI2 with `openni2::init()`,
//! then opening a `Device` and opening video `Stream`s. A stream can be configured
//! to record a desired `VideoMode`.
//!
//! From there, a program can synchronously read frames from a stream, or register
//! a callback to fire whenever a new frame is ready.
//!
//! ## Example
//!
//! ```no_run
//! extern crate openni2;
//!
//! use std::{thread, time};
//! use openni2::{Status, Device, Stream, SensorType, OniDepthPixel};
//!
//! fn callback(stream: &Stream<OniDepthPixel>) {
//!     // This function is only invoked when a frame *is* available to read
//!     let frame = stream.read_frame().expect("Frame not available to read!");
//!     let px = frame.pixels();
//!     let closest = px.iter()
//!         .enumerate()
//!         .fold((0u16, 0u16, ::std::u16::MAX), |closest, (n, &depth)| {
//!             let (x, y) = (n as u16 % frame.width(), n as u16 / frame.width());
//!             if depth < closest.2 && depth != 0 {
//!                 (x, y, depth)
//!             } else {
//!                 closest
//!             }
//!     });
//!     println!("[{:-6} {:-6} {:-6}]", closest.0, closest.1, closest.2);
//! }
//!
//! fn main() -> Result<(), Status> {
//!     // Initialize the library
//!     openni2::init()?;
//!
//!     // Open the first device we find, or abort early
//!     let device = Device::open_default()?;
//!
//!     // Get a handle for opening a stream from its depth sensor. If the device
//!     // didn't have a depth sensor, it would return `Err` and abort the program.
//!     let stream = device.create_stream(SensorType::DEPTH)?;
//!
//!     // Register a callback that will be called, with the stream as its first
//!     // argument, whenever a new frame is ready. When the listener falls out of
//!     // scope, the callback will be unregistered.
//!     let _listener = stream.listener(callback)?;
//!
//!     // Start the stream, then let the callback run until we kill the program
//!     // ourselves.
//!     stream.start()?;
//!
//!     let heartbeat = time::Duration::from_millis(250);
//!     loop {
//!         thread::sleep(heartbeat);
//!     }
//! }
//! ```

extern crate openni2_sys;
use std::os::raw::{c_int, c_void, c_char};
use std::ffi::{CString, CStr};
use std::{mem, ptr, slice};
use std::marker::PhantomData;
use openni2_sys::*;

mod device;
mod stream;
mod frame;
mod recorder;
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
    LogLevel,
};
pub use device::{Device, DeviceInfo};
pub use stream::{Stream, StreamListener, Cropping};
pub use frame::{Frame, frame_from_pointer};
pub use recorder::Recorder;
pub use openni2_sys::{
    OniDepthPixel,
    OniGrayscale16Pixel,
    OniGrayscale8Pixel,
    OniRGB888Pixel,
    OniYUV422DoublePixel,
};

/// Initialize the OpenNI2 library
pub fn init() -> Result<(), Status> {
    match unsafe { oniInitialize(2 * 1000 + 2) }.into() {
        Status::Ok => Ok(()),
        error => Err(error),
    }
}

#[doc(hidden)]
pub fn init_version(major: c_int, minor: c_int) -> Result<(), Status> {
    match unsafe { oniInitialize(major * 1000 + minor) }.into() {
        Status::Ok => Ok(()),
        error => Err(error),
    }
}

/// Shutdown the OpenNI2 library
pub fn shutdown() {
    unsafe { oniShutdown() };
}

/// Returns the version of the OpenNI2 library
///
/// ```
/// let version = openni2::get_version();
///
/// assert_eq!(2, version.major);
/// assert_eq!(2, version.minor);
/// assert_eq!(0, version.maintenance);
/// assert_eq!(33, version.build);
/// ```
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

/// Get a vector of `DeviceInfo` structs
///
/// ```
/// let devices = openni2::get_device_list();
///
/// for info in devices {
///     println!("{:?}", info);
/// }
/// ```
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

/// Register callbacks to execute whenever a device is connected, disconnected,
/// or changes state. The `DeviceInfo` that is passed in as the first argument
/// to the callbacks contains a `uri` field that can be used to open that
/// specific device.
///
/// See `device_callbacks` or `event_based_read` examples.
///
/// # Example
/// ```
/// # use openni2::{DeviceInfo, DeviceState, register_device_callbacks};
/// let mut connect = |device_info: DeviceInfo| {
///     println!("{} connected", device_info.uri);
/// };
///
/// let mut disconnect = |device_info: DeviceInfo| {
///     println!("{} disconnected", device_info.uri);
/// };

/// let mut state_change = |device_info: DeviceInfo, state: DeviceState| {
///    println!("{} changed state: {:?}", device_info.uri, state);
/// };
///
/// register_device_callbacks(connect, disconnect, state_change);
/// ```
///
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

/// Turn logging to console on or off.
pub fn set_console_log(state: bool) -> Status {
    unsafe {
        if state {
            oniSetLogConsoleOutput(1)
        } else {
            oniSetLogConsoleOutput(0)
        }
    }.into()
}

/// Turn logging to file on or off.
pub fn set_file_log(state: bool) -> Status {
    unsafe {
        if state {
            oniSetLogFileOutput(1)
        } else {
            oniSetLogFileOutput(0)
        }
    }.into()
}

/// Set the destination directory of the file log. Returns the name of the file, if successful.
/// If left unset, log files will be written to `"./Log"`
pub fn set_log_location(folder: &str) -> Result<(), Status> {
    if let Ok(path) = CString::new(folder) {
        let status = unsafe { oniSetLogOutputFolder(path.as_ptr()) }.into();
        match status {
            Status::Ok => Ok(()),
            _ => Err(status),
        }
    } else {
        Err(Status::Error(String::from("Invalid directory path")))
    }
}

/// Gets the filename of the current log file.
///
/// This will return an `Err` unless logging to file has already been turned on with `set_file_log`.
///
/// # Example
/// ```no_run
/// openni2::set_file_log(true);
/// openni2::set_log_location("./logs");
/// let file_name = openni2::get_log_file_name().unwrap();
/// // "/Users/toomanybees/code/rust-openni2/logs/2018_07_16__01_56_47_6304.log"
/// ```
pub fn get_log_file_name() -> Result<String, Status> {
    let mut buffer: [c_char; 256] = [0; 256];
    let status = unsafe { oniGetLogFileName(buffer.as_mut_ptr(), 256) }.into();
    println!("{:?}", status);
    match status {
        Status::Ok => Ok(unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_string_lossy().into_owned()),
        _ => Err(status),
    }
}

/// Set log level verbosity
pub fn set_log_level(severity: LogLevel) -> Status {
    unsafe { oniSetLogMinSeverity(severity as c_int) }.into()
}

/// When this falls out of scope, callbacks registered with `register_device_callbacks`
/// are unregistered.
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
