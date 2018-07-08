extern crate openni2;

use openni2::{Status, OniStreamHandle};
use std::os::raw::c_void;
use std::{thread, time, ptr};

unsafe extern "C" fn callback_function(
    _stream: OniStreamHandle,
    _cookie: *mut c_void) {
    println!("callback!");
}

fn main() {
    let version = openni2::get_version();
    openni2::init(version.major, version.minor);

    let mut d = openni2::Device::new();
    match d.open() {
        Status::Ok => {
            if let Ok(mut stream) = d.create_stream(openni2::SensorType::COLOR) {
                let ptr: unsafe extern "C" fn(OniStreamHandle, *mut c_void) -> () = callback_function;
                let callback = Some(ptr);
                let listener = stream.listener(callback, &mut ptr::null_mut() as &mut *mut i32);
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
