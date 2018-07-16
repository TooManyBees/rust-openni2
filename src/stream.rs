use std::marker::PhantomData;
use std::os::raw::{c_int, c_float, c_void};
use std::{ptr, fmt, mem, slice};

use openni2_sys::*;
use frame::Frame;
use types::{Status, SensorType, VideoMode, SensorInfo, Pixel, bytes_per_pixel};

pub struct Stream<'device, P: Pixel> {
    device_handle: &'device OniDeviceHandle,
    stream_handle: OniStreamHandle,
    sensor_type: SensorType,
    _pixel_type: PhantomData<P>,
}

impl<'device, P: Pixel> Stream<'device, P> {
    pub fn create(device_handle: &'device OniDeviceHandle, sensor_type: SensorType) -> Result<Self, Status> {
        let mut stream_handle: OniStreamHandle = ptr::null_mut();
        let status = unsafe {
            oniDeviceCreateStream(*device_handle, sensor_type as i32, &mut stream_handle)
        }.into();
        match status {
            Status::Ok => Ok(Stream {
                device_handle,
                stream_handle,
                sensor_type,
                _pixel_type: PhantomData,
            }),
            _ => Err(status)
        }
    }

    pub fn sensor_type(&self) -> SensorType {
        self.sensor_type
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

    pub fn get_cropping(&self) -> Result<Option<Cropping>, Status> {
        let oni_cropping = self.get_property::<OniCropping>(ONI_STREAM_PROPERTY_CROPPING)?;
        if oni_cropping.enabled > 0 {
            Ok(Some(Cropping {
                width: oni_cropping.width as u16,
                height: oni_cropping.height as u16,
                origin_x: oni_cropping.originX as u16,
                origin_y: oni_cropping.originY as u16,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn set_cropping(&self, value: Option<Cropping>) -> Result<(), Status> {
        let oni_cropping = match value {
            Some(cropping) => OniCropping {
                enabled: 1,
                width: cropping.width as c_int,
                height: cropping.height as c_int,
                originX: cropping.origin_x as c_int,
                originY: cropping.origin_y as c_int,
            },
            None => OniCropping {
                enabled: 0,
                width: 0,
                height: 0,
                originX: 0,
                originY: 0,
            },
        };
        self.set_property::<OniCropping>(ONI_STREAM_PROPERTY_CROPPING, oni_cropping)
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

    pub fn depth_to_world(&self, depth: (f32, f32, f32)) -> Result<(f32, f32, f32), Status> {
        // TODO: assert this is a depth stream
        let mut result = (0.0, 0.0, 0.0);
        let status = unsafe { oniCoordinateConverterDepthToWorld(self.stream_handle, depth.0, depth.1, depth.2, &mut result.0, &mut result.1, &mut result.2) }.into();
        if let Status::Ok = status {
            Ok(result)
        } else {
            Err(status)
        }
    }

    pub fn world_to_depth(&self, world: (f32, f32, f32)) -> Result<(f32, f32, f32), Status> {
        // TODO: assert this is a depth stream
        let mut result = (0.0, 0.0, 0.0);
        let status = unsafe { oniCoordinateConverterWorldToDepth(self.stream_handle, world.0, world.1, world.2, &mut result.0, &mut result.1, &mut result.2) }.into();
        if let Status::Ok = status {
            Ok(result)
        } else {
            Err(status)
        }
    }

    // todo: depth to color (requires 2 streams)

    pub fn reader(&self) -> StreamReader<P> {
        StreamReader { handle: &self.stream_handle, _pixel_type: PhantomData::<P> }
    }

    // Yowzers https://stackoverflow.com/questions/32270030/how-do-i-convert-a-rust-closure-to-a-c-style-callback
    pub fn listener<F: FnMut(&StreamReader<P>)>(&self, mut callback: F) -> Result<StreamListener<P>, Status> {
        let mut callback_handle: OniCallbackHandle = ptr::null_mut();

        // Ensure that pixel type P matches the pixel format that the
        // current video mode will return. Compile-time typing is possible
        // but is extremely impractical considering that a stream's video
        // mode can be changed.
        let type_param_size = mem::size_of::<P>();
        let pixel_size = {
            let video_mode = self.get_video_mode().expect("Couldn't fetch stream's video mode to assert that callback accepts a matching pixel format.");
            bytes_per_pixel(video_mode.pixel_format)
        };
        assert_eq!(type_param_size, pixel_size, "Size of callback's type parameter ({}) is different than the stream's pixel size reported by OpenNI2 ({}). Did you register the wrong callback on a stream?", type_param_size, pixel_size);

        extern "C" fn callback_wrapper(_: OniStreamHandle, cookie: *mut c_void) {
            let closure: &mut Box<FnMut()> = unsafe { mem::transmute(cookie) };
            closure();
        }

        let reader = self.reader();
        let closure: Box<Box<FnMut()>> = Box::new(Box::new(move || {
            callback(&reader);
        }));

        let status = unsafe {
            oniStreamRegisterNewFrameCallback(
                self.stream_handle,
                Some(callback_wrapper),
                Box::into_raw(closure) as *mut _,
                &mut callback_handle,
            )
        }.into();
        if let Status::Ok = status {
            Ok(StreamListener {
                stream_handle: &self.stream_handle,
                callback_handle,
                // closure_ptr,
                _pixel_type: PhantomData
            })
        } else {
            Err(status)
        }
    }
}

impl<'device, P: Pixel> fmt::Debug for Stream<'device, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Stream {{ device_handle: OniDeviceHandle({:p}), stream_handle: {:p} }}", self.device_handle, &self.stream_handle)
    }
}

impl<'device, P: Pixel> Drop for Stream<'device, P> {
    fn drop(&mut self) {
        // TODO: stop it too?
        // oniStreamDestroy doesn't return a status code :/
        unsafe { oniStreamDestroy(self.stream_handle) };
        mem::forget(self.stream_handle); // TODO: needed?
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Cropping {
    pub width: u16,
    pub height: u16,
    pub origin_x: u16,
    pub origin_y: u16,
}

pub struct StreamReader<'stream, P: Pixel> {
    handle: &'stream OniStreamHandle,
    _pixel_type: PhantomData<P>,
}

impl<'stream, P: Pixel> StreamReader<'stream, P> {
    pub fn read(&self) -> Frame<'stream, P> {
        let mut pointer = ptr::null_mut();
        let status = unsafe { oniStreamReadFrame(*self.handle, &mut pointer) }.into();
        match status {
            Status::Ok => {
                Frame::from_pointer(pointer)
            },
            _ => unreachable!(),
        }
    }

    // pub fn bytes_per_pixel(&self) -> usize {
    //     bytes_per_pixel(self.pixel_format)
    // }
}

pub struct StreamListener<'stream, P: Pixel> {
    stream_handle: &'stream OniStreamHandle,
    callback_handle: OniCallbackHandle,
    // closure_ptr: *mut c_void,
    _pixel_type: PhantomData<P>,
}

impl<'stream, P: Pixel> Drop for StreamListener<'stream, P> {
    fn drop(&mut self) {
        unsafe {
            oniStreamUnregisterNewFrameCallback(
                *self.stream_handle,
                self.callback_handle,
            );
        }
        // let _: Box<Box<FnMut()>> = unsafe { Box::from_raw(self.closure_ptr as *mut _) };
    }
}
