// Rust implementation of the OpenNI2 C++ sample
// https://github.com/OpenNI/OpenNI2/blob/master/Samples/MWClosestPoint/MWClosestPoint.cpp
// https://github.com/OpenNI/OpenNI2/blob/master/Samples/MWClosestPointApp/main.cpp
extern crate openni2;

use openni2::{Status, Stream};
use std::{thread, time};

fn callback(stream: &Stream<openni2::OniDepthPixel>) {
    // The whole idea of a stream listener is that its callback fires when a frame
    // is ready to read, thus the `expect` should be fine.
    let frame = stream.read_frame().expect("Frame somehow not available for read.");
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
    openni2::init()?;

    let device = openni2::Device::open_default()?;
    let stream = device.create_stream(openni2::SensorType::DEPTH)?;
    let _listener = stream.listener(callback)?;
    stream.start()?;

    let one_second = time::Duration::from_millis(1000);
    loop {
        thread::sleep(one_second);
    }
}
