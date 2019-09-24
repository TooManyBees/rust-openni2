use std::marker::PhantomData;
use std::os::raw::{c_int, c_float, c_void};
use std::{ptr, fmt, mem, slice};

use openni2_sys::*;
use device::Device;
use frame::{Frame, frame_from_pointer};
use types::{Status, SensorType, VideoMode, SensorInfo, Pixel};

/// A video stream that pulls frame from a single sensor on a `Device`.
///
/// The primary use of a `Stream` is to return `Frame` objects with the
/// `Stream::read_frame()` method. You need to specify both the `SensorType`
/// as an argument and the `Pixel` format as a type parameter.
///
/// # Example
/// ```no_run
/// # use openni2::{Device, SensorType, ColorPixelRGB888};
/// # fn main() -> Result<(), openni2::Status> {
/// let device = Device::open_default()?;
/// let stream = device.create_stream::<ColorPixelRGB888>(SensorType::COLOR)?;
/// stream.start()?;
/// loop {
///   let frame = stream.read_frame()?;
///   println!("{:?}", frame.pixels());
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Caveats
///
/// A `Stream` can only read frames while in its "started" state, set with
/// `Stream::start()`. A `Stream` can only change its video mode in its
/// "stopped" state, set with `Stream::stop()`. You don't need to worry
/// about manually stopping a stream before it falls out of scope.
pub struct Stream<'device, P: Pixel> {
    stream_handle: OniStreamHandle,
    sensor_type: SensorType,
    _pixel_type: PhantomData<&'device P>,
}

impl<'device, P: Pixel> Stream<'device, P> {
    #[doc(hidden)]
    pub(crate) fn create(device: &'device Device, sensor_type: SensorType) -> Result<Self, Status> {
        let mut stream_handle: OniStreamHandle = ptr::null_mut();
        let status = unsafe {
            oniDeviceCreateStream(device.handle, sensor_type as i32, &mut stream_handle)
        }.into();
        match status {
            Status::Ok => Ok(Stream {
                stream_handle,
                sensor_type,
                _pixel_type: PhantomData,
            }),
            _ => Err(status)
        }
    }

    #[doc(hidden)]
    pub fn handle(&self) -> OniStreamHandle {
        self.stream_handle
    }

    /// The `SensorType` that the stream is pulling frames from (color,
    /// depth, or IR).
    pub fn sensor_type(&self) -> SensorType {
        self.sensor_type
    }

    /// Starts the stream. If successful, the stream can then read
    /// frames from the device. Stop the stream with `Stream::stop`.
    pub fn start(&self) -> Result<(), Status> {
        let res = unsafe { oniStreamStart(self.stream_handle) }.into();
        match res {
            Status::Ok => Ok(()),
            _ => Err(res),
        }
    }

    /// Stops the stream. It can be restarted with `Stream::start` at any time.
    pub fn stop(&self) {
        unsafe { oniStreamStop(self.stream_handle) };
    }

    pub fn is_property_supported(&self, property: OniStreamProperty) -> bool {
        let res = unsafe { oniStreamIsPropertySupported(self.stream_handle, property) };
        res == 1
    }

    /// Return the stream's current `Cropping` which represents the
    /// subsection of the original video frame that this frame
    /// represents. Returns `None` if the frame is not cropped.
    ///
    /// # FIXME
    /// This method will return `Some(Cropping)` even if the crop
    /// is equal to the original frame dimensions, i.e. it is
    /// functionally uncropped.
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

    /// Sets the stream's crop to a specific `Cropping`, which describes the
    /// crop's width, height, and origin.
    ///
    /// # Example
    /// ```no_run
    /// # use openni2::{Device, SensorType, Cropping, ColorPixelRGB888};
    /// # fn main() -> Result<(), openni2::Status> {
    /// let device = Device::open_default()?;
    /// let stream = device.create_stream::<ColorPixelRGB888>(SensorType::COLOR)?;
    /// let crop = Cropping {
    ///     width: 100,
    ///     height: 100,
    ///     origin_x: 50,
    ///     origin_y: 50,
    /// };
    /// stream.set_cropping(Some(crop))?;
    /// stream.start()?;
    /// let frame = stream.read_frame()?;
    /// assert_eq!(frame.width(), crop.width);
    /// assert_eq!(frame.height(), crop.height);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_cropping(&self, value: Option<Cropping>) -> Result<(), Status> {
        let oni_cropping = match value {
            Some(cropping) => OniCropping {
                enabled: 1,
                width: c_int::from(cropping.width),
                height: c_int::from(cropping.height),
                originX: c_int::from(cropping.origin_x),
                originY: c_int::from(cropping.origin_y),
            },
            None => OniCropping {
                enabled: 0,
                width: 0,
                height: 0,
                originX: 0,
                originY: 0,
            },
        };
        self.set_property::<OniCropping>(ONI_STREAM_PROPERTY_CROPPING, &oni_cropping)
    }

    pub fn get_horizontal_fov(&self) -> Result<f32, Status> {
        self.get_property::<c_float>(ONI_STREAM_PROPERTY_HORIZONTAL_FOV)
    }

    pub fn get_vertical_fov(&self) -> Result<f32, Status> {
        self.get_property::<c_float>(ONI_STREAM_PROPERTY_VERTICAL_FOV)
    }

    /// Returns the current `VideoMode` of the stream, which includes
    /// the pixel format, the dimensions, and frame rate in FPS.
    pub fn get_video_mode(&self) -> Result<VideoMode, Status> {
        self.get_property::<OniVideoMode>(ONI_STREAM_PROPERTY_VIDEO_MODE)
        .map(|mode| {
            VideoMode {
                resolution_x: mode.resolutionX,
                resolution_y: mode.resolutionY,
                fps: mode.fps,
            }
        })
    }

    /// Sets a stream to a specific `VideoMode`. This will fail if the
    /// stream does not support such a video mode, or if the stream is
    /// started.
    pub fn set_video_mode(&self, value: VideoMode) -> Result<(), Status> {
        // TODO: validate dimensions and fps!
        let oni_value = OniVideoMode {
            pixelFormat: P::ONI_PIXEL_FORMAT,
            resolutionX: value.resolution_x,
            resolutionY: value.resolution_y,
            fps: value.fps,
        };
        self.set_property::<OniVideoMode>(ONI_STREAM_PROPERTY_VIDEO_MODE, &oni_value)
    }

    /// Returns the max possible numeric value of a depth pixel.
    /// For non-depth streams, this returns `None`.
    pub fn get_max_value(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_MAX_VALUE)
    }

    /// Returns the minimum possible numeric value of a depth pixel.
    /// For non-depth streams, this returns `None`.
    pub fn get_min_value(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_MIN_VALUE)
    }

    pub fn get_stride(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_STRIDE)
    }

    /// Returns whether the stream is currently mirrored.
    pub fn get_mirroring(&self) -> Result<bool, Status> {
        let res = self.get_property::<c_int>(ONI_STREAM_PROPERTY_MIRRORING)?;
        Ok(res == 1)
    }

    /// Set mirroring on the stream. This can be changed while the stream is
    /// running.
    pub fn set_mirroring(&self, value: bool) -> Result<(), Status> {
        self.set_property::<c_int>(ONI_STREAM_PROPERTY_MIRRORING, &(value as c_int))
    }

    pub fn get_number_of_frames(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_NUMBER_OF_FRAMES)
    }

    pub fn get_auto_white_balance(&self) -> Result<bool, Status> {
        let res = self.get_property::<c_int>(ONI_STREAM_PROPERTY_AUTO_WHITE_BALANCE)?;
        Ok(res == 1)
    }

    pub fn set_auto_white_balance(&self, value: bool) -> Result<(), Status> {
        self.set_property::<c_int>(ONI_STREAM_PROPERTY_AUTO_WHITE_BALANCE, &(value as c_int))
    }

    pub fn get_auto_exposure(&self) -> Result<bool, Status> {
        let res = self.get_property::<c_int>(ONI_STREAM_PROPERTY_AUTO_EXPOSURE)?;
        Ok(res == 1)
    }

    pub fn set_auto_exposure(&self, value: bool) -> Result<(), Status> {
        self.set_property::<c_int>(ONI_STREAM_PROPERTY_AUTO_EXPOSURE, &(value as c_int))
    }

    pub fn get_exposure(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_EXPOSURE)
    }

    // This gets truncated/wrapped to the range 0...65536 inclusive
    pub fn set_exposure(&self, value: i32) -> Result<(), Status> {
        self.set_property::<c_int>(ONI_STREAM_PROPERTY_EXPOSURE, &value)
    }

    pub fn get_gain(&self) -> Result<i32, Status> {
        self.get_property::<c_int>(ONI_STREAM_PROPERTY_GAIN)
    }

    // This gets truncated/wrapped to the range 0...65536 inclusive
    pub fn set_gain(&self, value: i32) -> Result<(), Status> {
        self.set_property::<c_int>(ONI_STREAM_PROPERTY_GAIN, &value)
    }

    fn get_property<T>(&self, property: OniStreamProperty) -> Result<T, Status> {
        let mut data = mem::MaybeUninit::<T>::uninit();
        let mut len = mem::size_of::<T>() as c_int;

        let status = unsafe {
            oniStreamGetProperty(
                self.stream_handle,
                property,
                data.as_mut_ptr() as *mut c_void,
                &mut len as *mut c_int,
            )
        }.into();

        match status {
            Status::Ok => Ok(unsafe { data.assume_init() }),
            _ => Err(status),
        }
    }

    fn set_property<T>(&self, property: OniStreamProperty, value: &T) -> Result<(), Status> {
        let len = mem::size_of::<T>() as c_int;
        let status = unsafe {
            oniStreamSetProperty(
                self.stream_handle,
                property,
                value as *const T as *const c_void,
                len,
            )
        }.into();

        match status {
            Status::Ok => Ok(()),
            _ => Err(status),
        }
    }

    /// Returns the stream's `SensorType` and a list of supported `VideoMode`s.
    ///
    /// # Example
    /// ```no_run
    /// # use openni2::{Device, SensorType, ColorPixelRGB888};
    /// # fn main() -> Result<(), openni2::Status> {
    /// let device = Device::open_default()?;
    /// let stream = device.create_stream::<ColorPixelRGB888>(SensorType::COLOR)?;
    /// let sensor_info = stream.sensor_info()?;
    /// let supported = sensor_info.video_modes.iter().find(|mode| {
    ///     mode.resolution_x == 640 &&
    ///     mode.resolution_y == 480 &&
    ///     mode.fps == 15
    /// });
    /// match supported {
    ///     Some(&video_mode) => stream.set_video_mode(video_mode)?,
    ///     None => panic!("Couldn't set desired video mode!"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn sensor_info(&self) -> Result<SensorInfo, Status> {
        unsafe {
            let ptr: *const OniSensorInfo = oniStreamGetSensorInfo(self.stream_handle);
            if ptr.is_null() {
                Err(Status::OutOfFlow)
            } else {
                let info: OniSensorInfo = *ptr;
                let len = info.numSupportedVideoModes as usize;
                assert!(!info.pSupportedVideoModes.is_null());
                let video_modes = slice::from_raw_parts(info.pSupportedVideoModes, len)
                    .iter()
                    .map(|&mode| mode.into())
                    .collect::<Vec<VideoMode>>();
                Ok(SensorInfo {
                    sensor_type: self.sensor_type,
                    video_modes,
                })
            }
        }
    }

    /// Reads the next `Frame` from the stream. This will block until a frame
    /// is ready. To avoid blocking, see the `Stream::listener` method to set
    /// a callback that will be invoked when a frame is ready to be read.
    ///
    /// # Example
    /// ```no_run
    /// # use openni2::{Device, SensorType, ColorPixelRGB888};
    /// # fn main() -> Result<(), openni2::Status> {
    /// let device = Device::open_default()?;
    /// let stream = device.create_stream::<ColorPixelRGB888>(SensorType::COLOR)?;
    /// let frame = stream.read_frame()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_frame(&self) -> Result<Frame<P>, Status> {
        let mut pointer = ptr::null_mut();
        let status = unsafe { oniStreamReadFrame(self.stream_handle, &mut pointer) }.into();
        match status {
            Status::Ok => unsafe { Ok(frame_from_pointer(pointer)) },
            _ => Err(status),
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

    /// Register a callback to execute when the stream has a frame immediately
    /// available.
    ///
    /// The callback function takes a single argument, which is the `Stream`
    /// itself. The callback can call `Stream::read_frame` on the stream,
    /// and it should not block.
    ///
    /// This method returns a `StreamListener` handle, which unregisters
    /// the callback when it falls out of scope.
    ///
    /// # Example
    /// ```no_run
    /// # use std::{thread, time};
    /// # use openni2::{Status, Device, Stream, SensorType, Pixel, DepthPixel1MM};
    /// # struct SomeHypotheticalDisplay {}
    /// # impl SomeHypotheticalDisplay {
    /// #     fn new() -> Self { SomeHypotheticalDisplay{} }
    /// #     fn update_from_buffer(&self, pixels: &[<DepthPixel1MM as Pixel>::Format]) {}
    /// # }
    /// fn main() -> Result<(), Status> {
    ///     openni2::init()?;
    ///     let device = Device::open_default()?;
    ///     let stream = device.create_stream(SensorType::DEPTH)?;
    ///     let display = SomeHypotheticalDisplay::new(); // pretend it's Minifb
    ///
    ///     let callback = |stream: &Stream<DepthPixel1MM>| {
    ///         let frame = stream.read_frame().unwrap();
    ///         display.update_from_buffer(&frame.pixels());
    ///     };
    ///
    ///     // Keep the callback registered until this falls out of scope
    ///     let _listener = stream.listener(&callback)?;
    ///
    ///     let one_second = time::Duration::from_millis(1000);
    ///     loop {
    ///         thread::sleep(one_second);
    ///     }
    ///     Ok(())
    /// # }
    /// ```
    pub fn listener<F: FnMut(&Stream<P>)>(&self, mut callback: F) -> Result<StreamListener, Status> {
        // Yowzers https://stackoverflow.com/questions/32270030/how-do-i-convert-a-rust-closure-to-a-c-style-callback
        let mut callback_handle: OniCallbackHandle = ptr::null_mut();

        extern "C" fn callback_wrapper(_: OniStreamHandle, cookie: *mut c_void) {
            // cookie, here, is a pointer to a Box<dyn FnMut()>
            let closure: &mut dyn FnMut() = unsafe { &mut *(cookie as *mut Box<dyn FnMut()>) };
            closure();
        }

        let closure: Box<Box<dyn FnMut()>> = Box::new(Box::new(move || {
            callback(&self);
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
                // _pixel_type: PhantomData
            })
        } else {
            Err(status)
        }
    }
}

impl<'device, P: Pixel> fmt::Debug for Stream<'device, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Stream {{ stream_handle: {:p} }}", &self.stream_handle)
    }
}

impl<'device, P: Pixel> Drop for Stream<'device, P> {
    fn drop(&mut self) {
        self.stop();
        unsafe { oniStreamDestroy(self.stream_handle) };
    }
}

/// Dimensions to crop a `Stream` to. See `Stream::set_cropping`
#[derive(Debug, Copy, Clone)]
pub struct Cropping {
    pub width: u16,
    pub height: u16,
    pub origin_x: u16,
    pub origin_y: u16,
}

/// Unregisters a `Stream`'s "new frame" callback when it falls
/// out of scope.
pub struct StreamListener<'stream> {
    stream_handle: &'stream OniStreamHandle,
    callback_handle: OniCallbackHandle,
    // closure_ptr: *mut c_void,
    // _pixel_type: PhantomData<P>,
}

impl<'stream> Drop for StreamListener<'stream> {
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
