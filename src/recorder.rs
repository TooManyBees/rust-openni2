use std::{fs, ptr};
use std::ffi::CString;
use std::path::Path;
use types::{Status, Pixel};
use stream::Stream;
use openni2_sys::*;

/// Records streams' frames to disk. After a `Recorder` is created, attach
/// `Stream`s to it through `Recorder::attach_stream(stream: &Stream)`.
/// Multiple streams can be attached at once. There is no requirement that
/// both the `Recorder` and `Stream` stay running continuously, but the
/// stream can not be attached while recording is running.
///
/// _See `examples/recording.rs` for an example._
#[derive(Debug)]
pub struct Recorder {
    handle: OniRecorderHandle
}

impl Recorder {
    /// Creates a new `Recorder` that will write frames to the file `filename`,
    /// which can be a relative path. The parent directories will be created if
    /// they do not already exist. If that fails, this method will return `Err`.
    pub fn create(filename: &str) -> Result<Recorder, Status> {
        let path = Path::new(filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| {
                Status::Error(format!("Couldn't create parent directory {:?}", parent))
            })?;
        }

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

    /// Attaches a stream to the recorder. The recorder will write frames as
    /// they are generated, once it is started. Streams can not be attached while
    /// the recorder is running.
    ///
    /// There is no practical limit to the number of streams that can be attached
    /// to the recorder at once.
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

    /// Begin recording. When attached `Stream`s produce frames, the frames will
    /// be written to the Recorder's file.
    pub fn start(&self) -> Result<(), Status> {
        match unsafe { oniRecorderStart(self.handle) }.into() {
            Status::Ok => Ok(()),
            status @ _ => Err(status),
        }
    }

    /// Stop recording. Recording can still be restarted with `Recorder::start`.
    pub fn stop(&self) {
        unsafe { oniRecorderStop(self.handle); }
    }

    /// Stops the recorder and destroys itself.
    pub fn close(self) {}
}

impl Drop for Recorder {
    fn drop(&mut self) {
        unsafe { oniRecorderDestroy(&mut self.handle) };
    }
}
