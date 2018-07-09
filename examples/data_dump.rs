extern crate openni2;
use openni2::{
    SensorType,
    Stream,
    OniRGB888Pixel,
    OniDepthPixel,
    OniGrayscale16Pixel,
    Pixel,
};

fn interrogate_stream<P: Pixel>(stream: &mut Stream<P>) {
    dump_stream_data(&stream);
    println!("Starting stream: {:?}", stream.start());
    {
        let stream_reader = stream.reader();
        for _ in 0..5 {
            let frame = stream_reader.read();
            println!("Got frame: {:?}", frame);
        }
    }
    stream.stop();
    println!("Stopping stream.");
}

fn dump_stream_data<P: Pixel>(stream: &Stream<P>) {
    println!("Cropping: {:?}", stream.get_cropping().ok());
    println!("Horizontal FOV: {:?}", stream.get_horizontal_fov().ok());
    println!("Vertical FOV: {:?}", stream.get_vertical_fov().ok());
    println!("Video Mode: {:?}", stream.get_video_mode().ok());
    println!("Max Value: {:?}", stream.get_max_value().ok());
    println!("Min Value: {:?}", stream.get_min_value().ok());
    println!("Stride: {:?}", stream.get_stride().ok());
    println!("Mirroring: {:?}", stream.get_mirroring().ok());
    println!("Number of frames: {:?}", stream.get_number_of_frames().ok());
    println!("Auto White Balance: {:?}", stream.get_auto_white_balance().ok());
    println!("Auto Exposure: {:?}", stream.get_auto_exposure().ok());
    println!("Auto Exposure: {:?}", stream.get_auto_exposure().ok());
    println!("Exposure: {:?}", stream.get_exposure().ok());
    println!("Gain: {:?}", stream.get_gain().ok());
}

fn main() {
    let version = openni2::get_version();
    openni2::init(version.major, version.minor);

    // openni2::set_console_log(true);

    // println!("{:?}", openni2::get_device_list());

    let mut d = openni2::Device::new();
    match d.open() {
        openni2::Status::Ok => {
            println!("{:?}", d.info());

            println!("Firmware Version: {:?}", d.get_firmware_version());
            println!("Driver Version: {:?}", d.get_driver_version());
            println!("Hardware Version: {:?}", d.get_hardware_version());
            println!("Serial No: {:?}", d.get_serial_number());

            if let Some(sensor_info) = d.get_sensor_info(SensorType::COLOR) {
                println!("{:#?}", sensor_info);
                if let Ok(mut stream) = d.create_stream::<OniRGB888Pixel>(SensorType::COLOR) {
                    interrogate_stream(&mut stream);
                }
            }

            if let Some(sensor_info) = d.get_sensor_info(SensorType::DEPTH) {
                println!("{:#?}", sensor_info);
                if let Ok(mut stream) = d.create_stream::<OniDepthPixel>(SensorType::DEPTH) {
                    interrogate_stream(&mut stream);
                }
            }

            if let Some(sensor_info) = d.get_sensor_info(SensorType::IR) {
                println!("{:#?}", sensor_info);
                if let Ok(mut stream) = d.create_stream::<OniGrayscale16Pixel>(SensorType::IR) {
                    interrogate_stream(&mut stream);
                }
            }
        },
        openni2::Status::Error(s) => println!("{}", s),
        e @ _ => println!("{:?}", e),
    }

    openni2::shutdown();
}
