extern crate openni2;

use openni2::{Status, StreamReader, Depth1MM};
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

    let mut d = openni2::Device::new();
    match d.open() {
        Status::Ok => {
            if let Ok(mut stream) = d.create_stream::<Depth1MM>() {
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
