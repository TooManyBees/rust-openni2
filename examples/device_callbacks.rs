extern crate openni2;
use openni2::{Status, DeviceInfo, DeviceState};
use std::{thread, time};

fn main() -> Result<(), Status> {
    let version = openni2::get_version();
    openni2::init(version.major, version.minor)?;

    let mut on_device_connect = |device_info: DeviceInfo| {
        println!("{} connected", device_info.uri);
    };

    let mut on_device_disconnect = |device_info: DeviceInfo| {
        println!("{} disconnected", device_info.uri);
    };

    let mut on_device_state_change = |device_info: DeviceInfo, state: DeviceState| {
        println!("{} changed state: {:?}", device_info.uri, state);
    };

    let _handle = openni2::register_device_callbacks(&mut on_device_connect, &mut on_device_disconnect, &mut on_device_state_change)?;

    let one_second = time::Duration::from_millis(1000);
    loop {
        thread::sleep(one_second);
    }
}
