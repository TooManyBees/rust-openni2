extern crate openni2;

use openni2::{Status, StreamReader};
use std::{thread, time};
use std::u16;

fn callback(reader: &StreamReader<openni2::OniDepthPixel>) {
    let frame = reader.read();
    let px = frame.pixels();
    let closest = px.iter()
        .enumerate()
        .fold((0u16, 0u16, u16::MAX), |closest, (n, &depth)| {
            let (x, y) = (n as u16 % frame.width, n as u16 / frame.width);
            if depth < closest.2 && depth != 0 {
                (x, y, depth)
            } else {
                closest
            }
    });
    println!("[{:-6} {:-6} {:-6}]", closest.0, closest.1, closest.2);
}

fn main() {
    let version = openni2::get_version();
    openni2::init(version.major, version.minor);

    match openni2::Device::open_default() {
        Ok(device) => {
            if let Ok(mut stream) = device.create_stream(openni2::SensorType::DEPTH) {
                let _listener = stream.listener(callback);
                stream.start();

                let one_second = time::Duration::from_millis(1000);
                loop {
                    thread::sleep(one_second);
                }
            }
        },
        _ => println!("Couldn't open device :(")
    }

    openni2::shutdown();
}
