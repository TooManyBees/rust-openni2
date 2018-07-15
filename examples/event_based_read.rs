// Rust implementation of the OpenNI2 C++ sample
// https://github.com/OpenNI/OpenNI2/blob/master/Samples/EventBasedRead/main.cpp

extern crate openni2;
use std::{thread, time};
use openni2::{
    Status,
    Device,
    DeviceInfo,
    DeviceState,
    SensorType,
    OniDepthPixel,
    StreamReader,
};

fn on_device_connect(device_info: DeviceInfo) {
    println!("{} connected", device_info.uri);
}

fn on_device_disconnect(device_info: DeviceInfo) {
    println!("{} disconnected", device_info.uri);
}

fn on_device_state_change(device_info: DeviceInfo, state: DeviceState) {
    println!("{} is now {:?}", device_info.uri, state);
}

fn main() -> Result<(), Status> {
    openni2::init(2, 2)?;

    openni2::register_device_callbacks(on_device_connect, on_device_disconnect, on_device_state_change)?;

    for device_info in openni2::get_device_list() {
        println!("{} already connected", device_info.uri);
    }

    let device = Device::open_default()?;
    let stream = device.create_stream(SensorType::DEPTH)?;

    let _listener = stream.listener(|reader: &StreamReader<OniDepthPixel>| {
        let frame = reader.read();
        let (width, height) = frame.dimensions();
        let px = frame.pixels();

        let middle = width as usize * height as usize / 2;

        println!("[{:08}]: {}x{} {:08}",
            frame.timestamp(), width, height, px[middle]);
    });

    stream.start();

    let one_second = time::Duration::from_millis(1000);
    loop {
        thread::sleep(one_second);
    }
}
