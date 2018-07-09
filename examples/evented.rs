extern crate openni2;

use openni2::{Status, StreamReader};
use std::{thread, time};

fn callback(reader: &StreamReader) {
    println!("graceful! {:?}", reader.read());
}

fn main() {
    let version = openni2::get_version();
    openni2::init(version.major, version.minor);

    let mut d = openni2::Device::new();
    match d.open() {
        Status::Ok => {
            if let Ok(mut stream) = d.create_stream(openni2::SensorType::COLOR) {
                let listener = stream.listener(callback);
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
