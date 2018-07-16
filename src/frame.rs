use openni2_sys::*;
use std::marker::PhantomData;
use types::{VideoMode, Pixel, bytes_per_pixel};
use std::{mem, slice};

#[derive(Debug)]
pub struct Frame<'a, T: Pixel> {
    oni_frame: &'a OniFrame,
    frame_pointer: *mut OniFrame,
    _pixel_type: PhantomData<T>,
}

impl<'a, T: Pixel> Frame<'a, T> {
    pub fn from_pointer(pointer: *mut OniFrame) -> Self {
        unsafe { oniFrameAddRef(pointer) };
        let oni_frame: &OniFrame = unsafe { &*pointer };
        Frame {
            oni_frame: oni_frame,
            frame_pointer: pointer,
            _pixel_type: PhantomData,
        }
    }

    pub fn timestamp(&self) -> u64 {
        self.oni_frame.timestamp
    }

    pub fn index(&self) -> usize {
        self.oni_frame.frameIndex as usize
    }

    pub fn width(&self) -> u16 {
        self.oni_frame.width as u16
    }

    pub fn height(&self) -> u16 {
        self.oni_frame.height as u16
    }

    pub fn video_mode(&self) -> VideoMode {
        self.oni_frame.videoMode.into()
    }

    pub fn cropped(&self) -> bool {
        self.oni_frame.croppingEnabled != 0
    }

    pub fn origin_x(&self) -> u16 {
        self.oni_frame.cropOriginX as u16
    }

    pub fn origin_y(&self) -> u16 {
        self.oni_frame.cropOriginY as u16
    }

    pub fn stride(&self) -> u16 {
        self.oni_frame.stride as u16
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

    pub fn dimensions(&self) -> (u16, u16) {
        (self.oni_frame.width as u16, self.oni_frame.height as u16)
    }

    pub fn inspect(&self) {
        println!("{:?}", self.oni_frame);
    }
}

impl<'a, P: Pixel> Drop for Frame<'a, P> {
    fn drop(&mut self) {
        mem::forget(self.oni_frame);
        unsafe { oniFrameRelease(self.frame_pointer); }
    }
}
