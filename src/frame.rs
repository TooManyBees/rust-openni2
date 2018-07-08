use openni2_sys::*;
use super::bytes_per_pixel;
use types::VideoMode;
use std::slice;

#[derive(Debug)]
pub struct Frame<'a> {
    oni_frame: &'a OniFrame,
    // frame_pointer: *mut OniFrame,
    pub width: u16,
    pub height: u16,
}

impl<'a> Frame<'a> {
    pub fn from_pointer(pointer: *mut OniFrame) -> Self {
        // unsafe { oniFrameAddRef(pointer) };
        let oni_frame: &OniFrame = unsafe { &*pointer };
        Frame {
            oni_frame: oni_frame,
            width: oni_frame.width as u16,
            height: oni_frame.height as u16,
            // frame_pointer: pointer,
        }
    }

    // FIXME: don't return private OniVideoMode
    pub fn video_mode(&self) -> VideoMode {
        self.oni_frame.videoMode.into()
    }

    pub fn inspect(&self) {
        let num_bytes = unsafe { oniFormatBytesPerPixel(self.oni_frame.videoMode.pixelFormat) };
        println!("{}", num_bytes);
    }

    pub fn pixels<T: Pixel>(&self) -> Vec<T> {
        let pixel_size = bytes_per_pixel(self.oni_frame.videoMode.pixelFormat.into());
        let num_pixels = (self.oni_frame.width * self.oni_frame.height) as usize;
        assert_eq!(self.oni_frame.dataSize as usize, num_pixels * pixel_size);
        let px = unsafe {
            slice::from_raw_parts(self.oni_frame.data as *const T, num_pixels)
        };
        px.to_vec()
    }
}

// impl<'a> Drop for Frame<'a> {
//     fn drop(&mut self) {
//         mem::forget(self.oni_frame);
//         unsafe { oniFrameRelease(self.frame_pointer); }
//     }
// }

macro_rules! isPixel {
    ($($in:ty),+) => (
        pub trait Pixel: Copy {}
        $(impl Pixel for $in {})+
    )
}
isPixel!(OniDepthPixel, /*OniGrayscale16Pixel,*/ OniGrayscale8Pixel, OniRGB888Pixel, OniYUV422DoublePixel);
