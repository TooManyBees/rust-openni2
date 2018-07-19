use std::ptr;
use std::ffi::CString;
use types::{Status, Pixel};
use stream::Stream;
use openni2_sys::*;

#[derive(Debug)]
pub struct Recorder {
    handle: OniRecorderHandle
}

impl Recorder {
    pub fn create(filename: &str) -> Result<Recorder, Status> {
        let mut handle = ptr::null_mut();
        let status = unsafe {
            let path = CString::new(filename).map_err(|_| {
                Status::Error(format!("Invalid filename: {:?}", filename))
            })?;
            oniCreateRecorder(path.as_ptr(), &mut handle)
        }.into();
        match status {
            Status::Ok => Ok(Recorder { handle }),
            _ => Err(status),
        }
    }

    // TODO: seems dangerous. What happens when recorder is destroyed with
    // streams attached to it?
    pub fn attach_stream<P: Pixel>(&self, stream: &Stream<P>, lossy: bool) -> Result<(), Status> {
        let lossy = if lossy { 1 } else { 0 };
        let status = unsafe {
            oniRecorderAttachStream(self.handle, stream.handle(), lossy)
        }.into();
        match status {
            Status::Ok => Ok(()),
            _ => Err(status),
        }
    }

    pub fn start(&self) -> Result<(), Status> {
        match unsafe { oniRecorderStart(self.handle) }.into() {
            Status::Ok => Ok(()),
            status @ _ => Err(status),
        }
    }

    pub fn stop(&self) {
        unsafe { oniRecorderStop(self.handle); }
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        unsafe { oniRecorderDestroy(&mut self.handle) };
    }
}
