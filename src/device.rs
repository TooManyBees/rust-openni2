use std::os::raw::{c_int, c_char, c_float, c_void};
use std::{ptr, fmt, mem, slice};
use std::ffi::{CString, CStr};

use openni2_sys::*;
use types::{Status, SensorType, ImageRegistrationMode, VideoMode, SensorInfo, Pixel};
use stream::Stream;

/// An open device. The device is closed when this struct drops out of scope.
pub struct Device {
    handle: OniDeviceHandle,
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Device {{ handle: OniDeviceHandle({:p}) }}", &self.handle)
    }
}

impl Device {
    /// Opens the first device that OpenNI2 can find.
    ///
    /// `Err` is returned when no devices are found.
    ///
    /// # Example
    /// ```no_run
    /// # use openni2::Device;
    /// # fn main() -> Result<(), openni2::Status> {
    /// let device = Device::open_default()?;
    /// println!("{}", device.info()?.name);
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_default() -> Result<Self, Status> {
        Self::open(None)
    }

    /// Open a specific device by its URI string. This can be obtained with the
    /// `openni2::get_device_list` function which returns `Vec<DeviceInfo>`.
    ///
    /// # Example
    /// ```no_run
    /// # use openni2::Device;
    /// # fn main() -> Result<(), openni2::Status> {
    /// let devices: Vec<Device> = openni2::get_device_list()
    ///     .into_iter()
    ///     .filter_map(|device_info| Device::open_uri(&device_info.uri).ok())
    ///     .collect();
    /// # Ok(())
    /// }
    /// ```
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

    /// Returns a `DeviceInfo` that describes the device.
    ///
    /// # Example
    /// ```no_run
    /// # use openni2::Device;
    /// # fn main() -> Result<(), openni2::Status> {
    /// let device = Device::open_default()?;
    /// let info = device.info()?;
    /// assert_eq!(&info.uri, "1d27/0601@20/2");
    /// assert_eq!(&info.vendor, "PrimeSense");
    /// assert_eq!(&info.name, "PS1080");
    /// assert_eq!(info.usb_vendor_id, 7463);
    /// assert_eq!(info.usb_product_id, 1537);
    /// # Ok(())
    /// # }
    pub fn info(&self) -> Result<DeviceInfo, Status> {
        let mut oni_info: OniDeviceInfo = unsafe { mem::uninitialized() };
        let status: Status = unsafe { oniDeviceGetInfo(self.handle, &mut oni_info) }.into();
        match status {
            Status::Ok => Ok(oni_info.into()),
            _ => Err(status),
        }
    }

    /// Returns a `SensorInfo` that describes a sensor.
    ///
    /// # Example
    /// ```no_run
    /// # fn main() -> Result<(), openni2::Status> {
    /// use openni2::{Device, SensorType};
    ///
    /// let device = Device::open_default()?;
    /// if let Some(sensor_info) = device.get_sensor_info(SensorType::COLOR) {
    ///     println!("{:?}", sensor_info);
    /// }
    /// # Ok(())
    /// # }
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
                Some(SensorInfo {
                    sensor_type,
                    video_modes,
                })
            }
        }
    }

    /// Creates a `Stream` for a given `SensorType`. The function must also be annotated
    /// with the type of pixel to return. Valid pixel types are `OniDepthPixel`,
    /// `OniGrayscale16Pixel`, `OniGrayscale8Pixel`, `OniRGB888Pixel`, and `OniYUV422DoublePixel`.
    ///
    /// # Example
    /// ```no_run
    /// use openni2::{Device, SensorType, OniRGB888Pixel, OniDepthPixel};
    /// # fn main() -> Result<(), openni2::Status> {
    /// let device = Device::open_default()?;
    /// let color = device.create_stream::<OniRGB888Pixel>(SensorType::COLOR)?;
    /// let depth = device.create_stream::<OniDepthPixel>(SensorType::DEPTH)?;
    /// # Ok(())
    /// # }
    pub fn create_stream<P: Pixel>(&self, sensor_type: SensorType) -> Result<Stream<P>, Status> {
        Stream::create(&self, sensor_type)
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

    #[doc(hidden)]
    pub fn is_property_supported(&self, property: OniDeviceProperty) -> bool {
        let res = unsafe { oniDeviceIsPropertySupported(self.handle, property) };
        res == 1
    }

    /// Returns the device's firmware version.
    /// # Example
    /// ```no_run
    /// # use openni2::{Device};
    /// # fn main() -> Result<(), openni2::Status> {
    /// let device = Device::open_default()?;
    /// let firmware_version = device.get_firmware_version()?;
    /// assert_eq!(&firmware_version, "5.8.22");
    /// # Ok(())
    /// # }
    pub fn get_firmware_version(&self) -> Result<String, Status> {
        let arr = self.get_property::<[c_char; ONI_MAX_STR as usize]>(ONI_DEVICE_PROPERTY_FIRMWARE_VERSION)?;
        let s = unsafe { CStr::from_ptr(arr.as_ptr()) }.to_string_lossy().into_owned();
        Ok(s)
    }

    /// Returns the device's driver version.
    /// # Example
    /// ```no_run
    /// # use openni2::{Device};
    /// # fn main() -> Result<(), openni2::Status> {
    /// let device = Device::open_default()?;
    /// let driver = device.get_driver_version()?;
    /// assert_eq!(driver.major, 5);
    /// assert_eq!(driver.minor, 1);
    /// assert_eq!(driver.maintenance, 4);
    /// assert_eq!(driver.build, 1);
    /// # Ok(())
    /// # }
    pub fn get_driver_version(&self) -> Result<OniVersion, Status> {
        // FIXME: don't return a private openni2-sys type
        self.get_property::<OniVersion>(ONI_DEVICE_PROPERTY_DRIVER_VERSION)
    }

    /// Returns the device's hardware version.
    /// # Example
    /// ```no_run
    /// # use openni2::{Device};
    /// # fn main() -> Result<(), openni2::Status> {
    /// let device = Device::open_default()?;
    /// let version = device.get_hardware_version()?;
    /// assert_eq!(version, 6);
    /// # Ok(())
    /// # }
    pub fn get_hardware_version(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_DEVICE_PROPERTY_HARDWARE_VERSION)
    }

    /// Returns the device's serial number.
    /// # Example
    /// ```no_run
    /// # use openni2::{Device};
    /// # fn main() -> Result<(), openni2::Status> {
    /// let device = Device::open_default()?;
    /// let serial = device.get_serial_number()?;
    /// assert_eq!(&serial, "1403180118");
    /// # Ok(())
    /// # }
    pub fn get_serial_number(&self) -> Result<String, Status> {
        let arr = self.get_property::<[c_char; ONI_MAX_STR as usize]>(ONI_DEVICE_PROPERTY_SERIAL_NUMBER)?;
        let s = unsafe { CStr::from_ptr(arr.as_ptr()) }.to_string_lossy().into_owned();
        Ok(s)
    }

    /// Returns whether the device supports an image registration mode. Different streams have
    /// slightly different fields of view. When image registration scales and translates them.
    /// The only modes supported are `ImageRegistrationMode::OFF` (do nothing), and
    /// `ImageRegistrationMode::DEPTH_TO_COLOR` (aligns depth and color streams).
    pub fn is_image_registration_mode_supported(&self, mode: ImageRegistrationMode) -> bool {
        unsafe { oniDeviceIsImageRegistrationModeSupported(self.handle, mode as i32) != 0 }
    }

    /// Returns whether image registration mode is set to resize and align depth and color streams.
    pub fn get_image_registration(&self) -> Result<bool, Status> {
        let res = self.get_property::<OniImageRegistrationMode>(ONI_DEVICE_PROPERTY_IMAGE_REGISTRATION)?;
        Ok(res == ONI_IMAGE_REGISTRATION_DEPTH_TO_COLOR)
    }

    /// Turns depth-to-color image registration on or off.
    pub fn set_image_registration(&self, on: bool) -> Result<(), Status> {
        self.set_property::<OniImageRegistrationMode>(
            ONI_DEVICE_PROPERTY_IMAGE_REGISTRATION,
            if on { ONI_IMAGE_REGISTRATION_DEPTH_TO_COLOR } else { ONI_IMAGE_REGISTRATION_OFF },
        )?;
        Ok(())
    }

    // DEVICE_PROPERTY_ERROR_STATE ??

    /// Gets the playback speed for recordings. In order for this method to
    /// work, the device must have been opened with a URI pointing to a
    /// recording file.
    pub fn get_playback_speed(&self) -> Result<f32, Status> {
        self.get_property::<c_float>(ONI_DEVICE_PROPERTY_PLAYBACK_SPEED)
    }

    /// Sets the playback speed for recordings.
    pub fn set_playback_speed(&self, value: f32) -> Result<(), Status> {
        self.set_property(ONI_DEVICE_PROPERTY_PLAYBACK_SPEED, value)
    }

    /// Returns whether playback repeat is turned on for a recording. In
    /// order fo this method to work, the device must have been opened with
    /// a URI pointing to a recording file.
    pub fn get_playback_repeat_enabled(&self) -> Result<bool, Status> {
        let res = self.get_property::<c_int>(ONI_DEVICE_PROPERTY_PLAYBACK_REPEAT_ENABLED)?;
        Ok(res == 1)
    }

    /// Sets the playback repeat for recordings.
    pub fn set_playback_repeat_enabled(&self, value: bool) -> Result<(), Status> {
        self.set_property(ONI_DEVICE_PROPERTY_PLAYBACK_REPEAT_ENABLED, value)
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

    fn set_property<T>(&self, property: OniDeviceProperty, value: T) -> Result<(), Status> {
        let len = mem::size_of::<T>() as c_int;
        let status = unsafe {
            oniDeviceSetProperty(
                self.handle,
                property,
                &value as *const T as *const _,
                len,
            )
        }.into();
        match status {
            Status::Ok => Ok(()),
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
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { oniDeviceClose(self.handle) };
    }
}

/// A descriptive information struct for a `Device`. Can be obtained for a specific device by
/// calling `Device::info(&self)`, or as a vector by calling `openni2::get_device_list()`.
#[derive(Debug)]
pub struct DeviceInfo {
    /// The identifying URI of the device. Can be passed to `Device::open_uri(&str)` to open
    /// the devcie described by this struct.
    pub uri: String,
    /// The vendor string of the device.
    pub vendor: String,
    /// The product name of the device.
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
