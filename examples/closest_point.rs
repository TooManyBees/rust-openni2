// Rust implementation of the OpenNI2 C++ sample
// https://github.com/OpenNI/OpenNI2/blob/master/Samples/MWClosestPoint/MWClosestPoint.cpp
// https://github.com/OpenNI/OpenNI2/blob/master/Samples/MWClosestPointApp/main.cpp
extern crate openni2;
use std::{thread, time};
use openni2::{Status, Device, Stream, SensorType, DepthPixel1MM};
fn callback(stream: &Stream<DepthPixel1MM>) {
    // This callback is only invoked when a frame *is* available to read,
    // so the `expect` is rather safe.
    let frame = stream.read_frame().expect("Frame not available to read!");
    let px = frame.pixels();
    // `DepthPixel1MM`'s associated type `<DepthPixel1MM as Pixel>::Format` tells us that this
    // stream will give us frames containing `u16` depth pixels.
    let closest = px.iter()
        .enumerate()
        .fold((0u16, 0u16, ::std::u16::MAX), |closest, (n, &depth)| {
            let (x, y) = (n as u16 % frame.width(), n as u16 / frame.width());
            if depth < closest.2 && depth != 0 {
                (x, y, depth)
            } else {
                closest
            }
        });
    println!("[{:-6} {:-6} {:-6}]", closest.0, closest.1, closest.2);
}
fn main() -> Result<(), Status> {
    // Initialize the library
    openni2::init()?;
    // Open the first device we find, or abort early
    let device = Device::open_default()?;
    // Get a handle for opening a stream from its depth sensor. If the device
    // didn't have a depth sensor, or if the depth sensor couldn't return this
    // particular format of pixel (a `u16` representing 1 millimeter of depth)
    // it would return `Err`.
    let stream = device.create_stream::<DepthPixel1MM>(SensorType::DEPTH)?;
    // Register a callback that will be called, with the stream as its first
    // argument, whenever a new frame is ready. When the listener falls out of
    // scope, the callback will be unregistered.
    let _listener = stream.listener(callback)?;
    // Start the stream, then let the callback run until we kill the program
    // ourselves.
    stream.start()?;
    let heartbeat = time::Duration::from_millis(250);
    loop {
        thread::sleep(heartbeat);
    }
}