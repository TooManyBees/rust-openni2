use openni2_sys::*;
use std::marker::PhantomData;
use types::{VideoMode, Pixel, bytes_per_pixel};
use std::{mem, slice};

#[doc(hidden)]
pub(crate) unsafe fn frame_from_pointer<P: Pixel>(frame_pointer: *mut OniFrame) -> Frame<P> {
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
/// # use openni2::{Device, ColorPixelRGB888, OniRGB888Pixel, SensorType};
/// # fn main() -> Result<(), openni2::Status> {
/// # let device = Device::open_default()?;
/// let stream = device.create_stream::<ColorPixelRGB888>(SensorType::COLOR)?;
/// let frame = stream.read_frame().unwrap();
/// assert_eq!(frame.width(), 320);
/// assert_eq!(frame.height(), 240);
/// assert_eq!(frame.pixels()[0], OniRGB888Pixel { r: 255, g: 173, b: 203 });
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

    /// Returns the actual pixel data of the frame as an array of pixels of type `P::Format`.
    ///
    /// # Panics
    ///
    /// If the byte size of the frame does not match the expected size for its dimensions and
    /// pixel format, this method will panic. This should only happen in the case of a malfunction
    /// of OpenNI2, or if a stream had its pixel format changed without its Rust type changing.
    ///
    /// (If it is possible to reach into OpenNI2 internals and unsafely change a stream's pixel
    /// format through the library, that is considered a bug.)
    pub fn pixels(&self) -> &[P::Format] {
        let num_pixels = self.oni_frame.width as usize * self.oni_frame.height as usize;

        if cfg!(debug_assertions) {
            assert_eq!(
                self.oni_frame.dataSize as usize,
                num_pixels * P::BYTES_PER_PIXEL,
                "Expected frame bytesize ({}) did not match actual pixel bytesize ({}).",
                self.oni_frame.dataSize,
                num_pixels * P::BYTES_PER_PIXEL
            );
        } else {
            assert_eq!(self.oni_frame.dataSize as usize, num_pixels * P::BYTES_PER_PIXEL, "Expected frame bytesize did not match actual pixel bytesize.");
        }
        unsafe {
            slice::from_raw_parts(self.oni_frame.data as *const P::Format, num_pixels)
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
