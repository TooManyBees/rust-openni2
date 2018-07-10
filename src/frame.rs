use openni2_sys::*;
use std::marker::PhantomData;
use super::bytes_per_pixel;
use types::{VideoMode, Pixel};
use std::{mem, slice};

#[derive(Debug)]
pub struct Frame<'a, T: Pixel> {
    oni_frame: &'a OniFrame,
    // frame_pointer: *mut OniFrame,
    pub width: u16,
    pub height: u16,
    _pixel_type: PhantomData<T>,
}

impl<'a, T: Pixel> Frame<'a, T> {
    pub fn from_pointer(pointer: *mut OniFrame) -> Self {
        // unsafe { oniFrameAddRef(pointer) };
        let oni_frame: &OniFrame = unsafe { &*pointer };
        Frame {
            oni_frame: oni_frame,
            width: oni_frame.width as u16,
            height: oni_frame.height as u16,
            // frame_pointer: pointer,
            _pixel_type: PhantomData,
        }
    }

    pub fn video_mode(&self) -> VideoMode {
        self.oni_frame.videoMode.into()
    }

    pub fn inspect(&self) {
        let num_bytes = unsafe { oniFormatBytesPerPixel(self.oni_frame.videoMode.pixelFormat) };
        println!("{}", num_bytes);
    }

    pub fn pixels(&self) -> &[T] {
        let pixel_size = bytes_per_pixel(self.oni_frame.videoMode.pixelFormat.into());
        let type_param_size = mem::size_of::<T>();
        assert_eq!(type_param_size, pixel_size, "Size of Frame::pixels() type parameter ({}) is different than pixel size reported by OpenNI2 ({}). If this method worked before, you may have changed the video mode of a stream without unregistering an existing callback.", type_param_size, pixel_size);

        let num_pixels = self.oni_frame.width as usize * self.oni_frame.height as usize;
        assert_eq!(self.oni_frame.dataSize as usize, num_pixels * pixel_size);
        unsafe {
            slice::from_raw_parts(self.oni_frame.data as *const T, num_pixels)
        }
    }

    pub fn pixels_owned(&self) -> Vec<T> {
        self.pixels().to_vec()
    }
}

// impl<'a> Drop for Frame<'a> {
//     fn drop(&mut self) {
//         mem::forget(self.oni_frame);
//         unsafe { oniFrameRelease(self.frame_pointer); }
//     }
// }
