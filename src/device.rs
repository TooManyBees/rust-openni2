use std::os::raw::{c_int, c_char, c_float, c_void};
use std::{ptr, fmt, mem, slice};
use std::ffi::{CString, CStr};

use openni2_sys::*;
use types::{Status, SensorType, ImageRegistrationMode, VideoMode, SensorInfo, Pixel};
use stream::Stream;

pub struct Device {
    handle: OniDeviceHandle,
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Device {{ handle: OniDeviceHandle({:p}) }}", &self.handle)
    }
}

impl Device {
    pub fn open_default() -> Result<Self, Status> {
        Self::open(None)
    }

    pub fn open_uri(uri: &str) -> Result<Self, Status> {
        let cstring = CString::new(uri);
        match cstring {
            Ok(cstring) => Self::open(Some(cstring)),
            Err(_) => Err(Status::Error(format!("Uri `{}` was not a valid CString", uri))),
        }
    }

    fn open(uri: Option<CString>) -> Result<Self, Status> {
        let mut handle = ptr::null_mut();
        // Careful not to `match uri` without borrowing,
        // the cstring will get moved out of the Option
        // and dropped as a dangling pointer
        let uri_ptr = match &uri {
            Some(cstring) => cstring.as_ptr(),
            None => ptr::null(),
        };
        let status = unsafe { oniDeviceOpen(uri_ptr, &mut handle ) }.into();
        match status {
            Status::Ok => Ok(Device { handle }),
            _ => Err(status),
        }
    }

    pub fn info(&self) -> DeviceInfo {
        let mut oni_info: OniDeviceInfo = unsafe { mem::uninitialized() };
        let status: Status = unsafe { oniDeviceGetInfo(self.handle, &mut oni_info) }.into();
        match status {
            Status::Ok => {
                oni_info.into()
            },
            _ => {
                // mem::forget(oni_info);
                panic!("Couldn't get device info; add error handling!");
            }
        }
    }

    pub fn get_sensor_info(&self, sensor_type: SensorType) -> Option<SensorInfo> {
        unsafe {
            let ptr: *const OniSensorInfo = oniDeviceGetSensorInfo(self.handle, sensor_type as i32);
            if ptr.is_null() {
                None
            } else {
                let info: OniSensorInfo = *ptr;
                let len = info.numSupportedVideoModes as usize;
                assert!(!info.pSupportedVideoModes.is_null());
                let video_modes = slice::from_raw_parts(info.pSupportedVideoModes, len)
                    .iter()
                    .map(|&mode| mode.into())
                    .collect::<Vec<VideoMode>>();
                mem::forget(info); // i think?
                Some(SensorInfo {
                    sensor_type: sensor_type,
                    video_modes: video_modes,
                })
            }
        }
    }

    pub fn create_stream<P: Pixel>(&self, sensor_type: SensorType) -> Result<Stream<P>, Status> {
        Stream::create(&self.handle, sensor_type)
    }

    pub fn color_depth_sync(&self) -> bool {
        unsafe {
            oniDeviceGetDepthColorSyncEnabled(self.handle) != 0
        }
    }

    pub fn enable_color_depth_sync(&mut self, enabled: bool) -> Status {
        if enabled {
            unsafe { oniDeviceEnableDepthColorSync(self.handle) }.into()
        } else {
            unsafe { oniDeviceDisableDepthColorSync(self.handle) }
            Status::Ok
        }
    }

    pub fn is_property_supported(&self, property: OniDeviceProperty) -> bool {
        let res = unsafe { oniDeviceIsPropertySupported(self.handle, property) };
        res == 1
    }

    pub fn get_firmware_version(&self) -> Result<CString, Status> {
        let arr = self.get_property::<[c_char; ONI_MAX_STR as usize]>(ONI_DEVICE_PROPERTY_FIRMWARE_VERSION)?;
        let s = CString::new(arr.iter().take_while(|&c| *c != 0).map(|&c| c as u8).collect::<Vec<u8>>())
            .expect("CString::new failed");
        Ok(s)
    }

    pub fn get_driver_version(&self) -> Result<OniVersion, Status> {
        self.get_property::<OniVersion>(ONI_DEVICE_PROPERTY_DRIVER_VERSION)
    }

    pub fn get_hardware_version(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_DEVICE_PROPERTY_HARDWARE_VERSION)
    }

    pub fn get_serial_number(&self) -> Result<CString, Status> {
        let arr = self.get_property::<[c_char; ONI_MAX_STR as usize]>(ONI_DEVICE_PROPERTY_SERIAL_NUMBER)?;
        let s = CString::new(arr.iter().take_while(|&c| *c != 0).map(|&c| c as u8).collect::<Vec<u8>>())
            .expect("CString::new failed");
        Ok(s)
    }

    pub fn get_image_registration(&self) -> Result<bool, Status> {
        let res = self.get_property::<OniImageRegistrationMode>(ONI_DEVICE_PROPERTY_IMAGE_REGISTRATION)?;
        Ok(res == ONI_IMAGE_REGISTRATION_DEPTH_TO_COLOR)
    }

    // DEVICE_PROPERTY_ERROR_STATE ??

    pub fn get_playback_speed(&self) -> Result<f32, Status> {
        self.get_property::<c_float>(ONI_DEVICE_PROPERTY_PLAYBACK_SPEED)
    }

    pub fn get_playback_repeat_enabled(&self) -> Result<bool, Status> {
        let res = self.get_property::<c_int>(ONI_DEVICE_PROPERTY_PLAYBACK_REPEAT_ENABLED)?;
        Ok(res == 1)
    }

    fn get_property<T>(&self, property: OniDeviceProperty) -> Result<T, Status> {
        let mut data: T = unsafe { mem::uninitialized() };
        let mut len = mem::size_of::<T>() as c_int;

        let status = unsafe {
            oniDeviceGetProperty(
                self.handle,
                property, &mut data as *mut _ as *mut c_void,
                &mut len as *mut c_int,
            )
        }.into();

        match status {
            Status::Ok => Ok(data),
            _ => Err(status),
        }
    }

    // TODO: just replace with seek??
    // pub fn is_command_supported(&self, command: Command) -> bool {
    //     unsafe { oniDeviceIsCommandSupported(self.handle, command) != 0 }
    // }

    // pub fn invoke_command(&mut self, command: Command) -> Status {
    //     unsafe { oniDeviceInvoke(self.handle, command, data, dataSize) }.into()
    // }

    pub fn is_image_registration_mode_supported(&self, mode: ImageRegistrationMode) -> bool {
        unsafe { oniDeviceIsImageRegistrationModeSupported(self.handle, mode as i32) != 0 }
    }


}

impl Drop for Device {
    fn drop(&mut self) {
        let status = unsafe { oniDeviceClose(self.handle) }.into();
        if let Status::Ok = status {
            mem::forget(self.handle);
        }
    }
}

#[derive(Debug)]
pub struct DeviceInfo {
    pub uri: String,
    pub vendor: String,
    pub name: String,
    pub usb_vendor_id: u16,
    pub usb_product_id: u16,
}

impl From<OniDeviceInfo> for DeviceInfo {
    fn from(info: OniDeviceInfo) -> Self {
        DeviceInfo {
            uri: unsafe { CStr::from_ptr(info.uri.as_ptr()) }.to_string_lossy().into_owned(),
            vendor: unsafe { CStr::from_ptr(info.vendor.as_ptr()) }.to_string_lossy().into_owned(),
            name: unsafe { CStr::from_ptr(info.name.as_ptr()) }.to_string_lossy().into_owned(),
            usb_vendor_id: info.usbVendorId as u16,
            usb_product_id: info.usbProductId as u16,
        }
    }
}
