use std::marker::PhantomData;
use std::os::raw::{c_int, c_float, c_void};
use std::{ptr, fmt, mem, slice};

use openni2_sys::*;
use frame::Frame;
use types::{Status, SensorType, PixelFormat, VideoMode, SensorInfo};
use super::bytes_per_pixel;

pub struct Stream<'device> {
    device_handle: &'device OniDeviceHandle,
    stream_handle: OniStreamHandle,
    sensor_type: SensorType,
}

impl<'device> Stream<'device> {
    pub fn create(device_handle: &'device OniDeviceHandle, sensor_type: SensorType) -> Result<Self, Status> {
        let mut stream_handle: OniStreamHandle = ptr::null_mut();
        let status = unsafe {
            oniDeviceCreateStream(*device_handle, sensor_type as i32, &mut stream_handle)
        }.into();
        match status {
            Status::Ok => Ok(Stream {
                device_handle: device_handle,
                stream_handle: stream_handle,
                sensor_type: sensor_type,
            }),
            _ => Err(status)
        }
    }

    pub fn start(&self) -> Status {
        unsafe { oniStreamStart(self.stream_handle) }.into()
    }

    pub fn stop(&self) {
        unsafe { oniStreamStop(self.stream_handle) };
    }

    pub fn is_property_supported(&self, property: OniStreamProperty) -> bool {
        let res = unsafe { oniStreamIsPropertySupported(self.stream_handle, property) };
        res == 1
    }

    pub fn get_cropping(&self) -> Result<OniCropping, Status> {
        self.get_property::<OniCropping>(ONI_STREAM_PROPERTY_CROPPING)
    }

    pub fn set_cropping(&self, value: OniCropping) -> Result<(), Status> {
        self.set_property::<OniCropping>(ONI_STREAM_PROPERTY_CROPPING, value)
    }

    pub fn get_horizontal_fov(&self) -> Result<f32, Status> {
        self.get_property::<c_float>(ONI_STREAM_PROPERTY_HORIZONTAL_FOV)
    }

    pub fn get_vertical_fov(&self) -> Result<f32, Status> {
        self.get_property::<c_float>(ONI_STREAM_PROPERTY_VERTICAL_FOV)
    }

    pub fn get_video_mode(&self) -> Result<VideoMode, Status> {
        self.get_property::<OniVideoMode>(ONI_STREAM_PROPERTY_VIDEO_MODE)
        .map(|mode| {
            VideoMode {
                pixel_format: mode.pixelFormat.into(),
                resolution_x: mode.resolutionX,
                resolution_y: mode.resolutionY,
                fps: mode.fps,
            }
        })
    }

    pub fn set_video_mode(&self, value: VideoMode) -> Result<(), Status> {
        // TODO: validate dimensions and fps!
        let oni_value = OniVideoMode {
            pixelFormat: value.pixel_format as OniPixelFormat,
            resolutionX: value.resolution_x,
            resolutionY: value.resolution_y,
            fps: value.fps,
        };
        self.set_property::<OniVideoMode>(ONI_STREAM_PROPERTY_VIDEO_MODE, oni_value)
    }

    pub fn get_max_value(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_MAX_VALUE)
    }

    pub fn get_min_value(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_MIN_VALUE)
    }

    pub fn get_stride(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_STRIDE)
    }

    pub fn get_mirroring(&self) -> Result<bool, Status> {
        let res = self.get_property::<c_int>(ONI_STREAM_PROPERTY_MIRRORING)?;
        Ok(res == 1)
    }

    pub fn set_mirroring(&self, value: bool) -> Result<(), Status> {
        self.set_property::<c_int>(ONI_STREAM_PROPERTY_MIRRORING, value as c_int)
    }

    pub fn get_number_of_frames(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_NUMBER_OF_FRAMES)
    }

    pub fn get_auto_white_balance(&self) -> Result<bool, Status> {
        let res = self.get_property::<c_int>(ONI_STREAM_PROPERTY_AUTO_WHITE_BALANCE)?;
        Ok(res == 1)
    }

    pub fn set_auto_white_balance(&self, value: bool) -> Result<(), Status> {
        self.set_property::<c_int>(ONI_STREAM_PROPERTY_AUTO_WHITE_BALANCE, value as c_int)
    }

    pub fn get_auto_exposure(&self) -> Result<bool, Status> {
        let res = self.get_property::<c_int>(ONI_STREAM_PROPERTY_AUTO_EXPOSURE)?;
        Ok(res == 1)
    }

    pub fn set_auto_exposure(&self, value: bool) -> Result<(), Status> {
        self.set_property::<c_int>(ONI_STREAM_PROPERTY_AUTO_EXPOSURE, value as c_int)
    }

    pub fn get_exposure(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_EXPOSURE)
    }

    // This gets truncated/wrapped to the range 0...65536 inclusive
    pub fn set_exposure(&self, value: i32) -> Result<(), Status> {
        self.set_property::<c_int>(ONI_STREAM_PROPERTY_EXPOSURE, value)
    }

    pub fn get_gain(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_GAIN)
    }

    // This gets truncated/wrapped to the range 0...65536 inclusive
    pub fn set_gain(&self, value: i32) -> Result<(), Status> {
        self.set_property::<c_int>(ONI_STREAM_PROPERTY_GAIN, value)
    }

    fn get_property<T>(&self, property: OniStreamProperty) -> Result<T, Status> {
        let mut data: T = unsafe { mem::uninitialized() };
        let mut len = mem::size_of::<T>() as c_int;

        let status = unsafe {
            oniStreamGetProperty(
                self.stream_handle,
                property,
                &mut data as *mut _ as *mut c_void,
                &mut len as *mut c_int,
            )
        }.into();

        match status {
            Status::Ok => Ok(data),
            _ => Err(status),
        }
    }

    fn set_property<T>(&self, property: OniStreamProperty, value: T) -> Result<(), Status> {
        let len = mem::size_of::<T>() as c_int;
        let status = unsafe {
            oniStreamSetProperty(
                self.stream_handle,
                property,
                &value as *const T as *const c_void,
                len,
            )
        }.into();

        match status {
            Status::Ok => Ok(()),
            _ => Err(status),
        }
    }

    pub fn get_sensor_info(&self) -> Option<SensorInfo> {
        unsafe {
            let ptr: *const OniSensorInfo = oniStreamGetSensorInfo(self.stream_handle);
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
                    sensor_type: self.sensor_type,
                    video_modes: video_modes,
                })
            }
        }
    }

    // pub fn is_command_supported(&self, command: OniStreamCommand) -> bool {
    //     let res = unsafe { oniStreamIsCommandSupported(self.stream_handle, command) }
    //     res == 1
    // }

    pub fn reader(&mut self /*, pixel_format: PixelFormat */) -> StreamReader {
        let video_format = self.get_video_mode()
            .expect("couldn't check video format of stream before reading");
        StreamReader { handle: &self.stream_handle, pixel_format: video_format.pixel_format }
    }
}

impl<'device> fmt::Debug for Stream<'device> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Stream {{ device_handle: OniDeviceHandle({:p}), stream_handle: {:p} }}", self.device_handle, &self.stream_handle)
    }
}

impl<'device> Drop for Stream<'device> {
    fn drop(&mut self) {
        // oniStreamDestroy doesn't return a status code :/
        unsafe { oniStreamDestroy(self.stream_handle) };
        mem::forget(self.stream_handle);
    }
}

pub struct StreamReader<'stream> {
    handle: &'stream OniStreamHandle,
    pixel_format: PixelFormat,
}

impl<'stream> StreamReader<'stream> {
    pub fn read(&self) -> Frame<'stream> {
        let mut pointer = ptr::null_mut();
        let status = unsafe { oniStreamReadFrame(*self.handle, &mut pointer) }.into();
        match status {
            Status::Ok => {
                Frame::from_pointer(pointer)
            },
            _ => unreachable!(),
        }
    }

    pub fn bytes_per_pixel(&self) -> usize {
        bytes_per_pixel(self.pixel_format)
    }
}
