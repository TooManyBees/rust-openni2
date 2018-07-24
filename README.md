# OpenNI2 for Rust

In-development Rust wrapper for [OpenNI2](https://github.com/occipital/OpenNI2).
OpenNI2 is useful for working with multi-sensor cameras that can simultaneously
serve color and depth streams, particularly sensors developed by PrimeSense
(a founding member of the OpenNI software project) such as the Xbox Kinect,
and ASUS Xtion.

# App example

```rust
extern crate openni2;
use std::{thread, time};
use openni2::{Status, Device, Stream, SensorType, OniDepthPixel};
fn callback(stream: &Stream<OniDepthPixel>) {
    // This function is only invoked when a frame *is* available to read
    let frame = stream.read_frame().expect("Frame not available to read!");
    let px = frame.pixels();
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
    // didn't have a depth sensor, it would return `Err` and abort the program.
    let stream = device.create_stream(SensorType::DEPTH)?;
    // Register a callback that will be called, with the stream as its first
    // argument, whenever a new frame is ready. When the listener falls out of
    // scope, the callback will be unregistered.
    let _listener = stream.listener(callback)?;
    // Start the stream, then let the callback run until we kill the program
    // ourselves.
    stream.start();
    let heartbeat = time::Duration::from_millis(250);
    loop {
        thread::sleep(heartbeat);
    }
}
```

# Examples

[`examples/data_dump.rs`](examples/data_dump.rs) demonstrates interrogating
devices and streams about their properties, as well as blocking for new frames.

[`examples/closest_point.rs`](examples/closest_point.rs) demonstrates event-based
callbacks, and finding the closest point in a depth map.

[`examples/device_callbacks.rs`](examples/device_callbacks.rs) demonstrates device callbacks that detect newly connected/disconnected devices

[`examples/simple_viewer.rs`](examples/simple_viewer.rs) is a video stream viewer with keyboard controls.
* `1` views the color and depth streams overlayed
* `2` views the color stream
* `3` views the depth stream
* `m` toggles video stream mirroring
