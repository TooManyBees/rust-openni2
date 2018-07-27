use openni2_sys::*;
use std::marker::PhantomData;
use types::{VideoMode, Pixel, bytes_per_pixel};
use std::{mem, slice};

#[doc(hidden)]
pub unsafe fn frame_from_pointer<P: Pixel>(frame_pointer: *mut OniFrame) -> Frame<P> {
    assert!(!frame_pointer.is_null());
    let oni_frame: OniFrame = *frame_pointer;
    Frame {
        oni_frame,
        frame_pointer,
        _pixel_type: PhantomData,
    }
}

/// A single frame of video data.
///
/// # Example
///
/// ```no_run
/// # use openni2::{Device, OniRGB888Pixel, SensorType};
/// # fn main() -> Result<(), openni2::Status> {
/// # let device = Device::open_default()?;
/// let stream = device.create_stream::<OniRGB888Pixel>(SensorType::COLOR)?;
/// let frame = stream.read_frame().unwrap();
/// assert_eq!(frame.width(), 320);
/// assert_eq!(frame.height(), 240);
/// println!("{:?}", frame.pixels()[0]); // "OniRGB888Pixel { r: 255, g: 173, b: 203 }"
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Frame<P: Pixel> {
    oni_frame: OniFrame,
    frame_pointer: *mut OniFrame,
    _pixel_type: PhantomData<P>,
}

impl<P: Pixel> Frame<P> {
    /// The timestamp of the frame.
    pub fn timestamp(&self) -> u64 {
        self.oni_frame.timestamp
    }

    /// The sequential index of the frame.
    pub fn index(&self) -> usize {
        self.oni_frame.frameIndex as usize
    }

    /// The width of the frame.
    pub fn width(&self) -> u16 {
        self.oni_frame.width as u16
    }

    /// The height of the frame.
    pub fn height(&self) -> u16 {
        self.oni_frame.height as u16
    }

    /// Returns the `VideoMode` of the frame, which describes the pixel format, dimensions,
    /// and frame rate of the video stream that produced it.
    pub fn video_mode(&self) -> VideoMode {
        self.oni_frame.videoMode.into()
    }

    /// Returns true if the frame is cropped. If true, the `width` and `height` methods
    /// represent the cropped dimensions.
    pub fn cropped(&self) -> bool {
        self.oni_frame.croppingEnabled != 0
    }

    /// The left offset of the cropped frame. An uncropped frame has an x origin of 0.
    pub fn origin_x(&self) -> u16 {
        self.oni_frame.cropOriginX as u16
    }

    /// The top offset of the cropped frame. An uncropped frame has a y origin of 0.
    pub fn origin_y(&self) -> u16 {
        self.oni_frame.cropOriginY as u16
    }

    /// The number of bytes in a row of frame data. (`width * size_of::<Pixel>()`)
    pub fn stride(&self) -> u16 {
        self.oni_frame.stride as u16
    }

    /// Returns the actual pixel data of the frame as an array of pixels of type `P`.
    ///
    /// # Panics
    /// `Frame::pixels` will panic if the byte size of the pixel format, as described in the
    /// frame's `VideoMode`, doesn't match `mem::size_of::<P>()`. This could happen if you
    /// created a stream with a `Pixel` type parameter that doesn't match what the stream
    /// is actually going to return.
    ///
    /// In other word's it's the programmer's responsibility to type the `Stream` correctly.
    pub fn pixels(&self) -> &[P] {
        let pixel_size = bytes_per_pixel(self.oni_frame.videoMode.pixelFormat.into());
        let type_param_size = mem::size_of::<P>();
        assert_eq!(type_param_size, pixel_size, "Size of Frame::pixels() type parameter ({}) is different than pixel size reported by OpenNI2 ({}). If this method worked before, you may have changed the video mode of a stream without unregistering an existing callback.", type_param_size, pixel_size);

        let num_pixels = self.oni_frame.width as usize * self.oni_frame.height as usize;
        assert_eq!(self.oni_frame.dataSize as usize, num_pixels * pixel_size);
        unsafe {
            slice::from_raw_parts(self.oni_frame.data as *const P, num_pixels)
        }
    }

    /// A shorthand method for frame width and height.
    pub fn dimensions(&self) -> (u16, u16) {
        (self.oni_frame.width as u16, self.oni_frame.height as u16)
    }

    #[doc(hidden)]
    pub fn inspect(&self) {
        println!("{:?}", self.oni_frame);
    }
}

impl<P: Pixel> Drop for Frame<P> {
    fn drop(&mut self) {
        unsafe { oniFrameRelease(self.frame_pointer); }
    }
}

impl<P: Pixel> Clone for Frame<P> {
    fn clone(&self) -> Frame<P> {
        unsafe { oniFrameAddRef(self.frame_pointer); }
        Frame {
            oni_frame: self.oni_frame,
            frame_pointer: self.frame_pointer,
            _pixel_type: PhantomData,
        }
    }
}
