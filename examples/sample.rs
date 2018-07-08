extern crate openni2;

fn main() {
    let version = openni2::get_version();
    openni2::init(version.major, version.minor);
    openni2::set_console_log(true);

    let devices = openni2::get_device_list();
    println!("{:?}", devices);

    let mut d = openni2::Device::new();
    println!("{:?}", &d);
    match d.open() {
        openni2::Status::Ok => {
            println!("Yay, we opened it!");

            let info = d.info();
            println!("{:?}", info);

            // println!("Firmware Version: {:?}", d.get_firmware_version());
            // println!("Driver Version: {:?}", d.get_driver_version());
            // println!("Hardware Version: {:?}", d.get_hardware_version());
            // println!("Serial No: {:?}", d.get_serial_number());

            if let Some(sensor_info) = d.get_sensor_info(openni2::SensorType::COLOR) {
                println!("{:?}", sensor_info);
                if let Ok(mut stream) = d.create_stream(openni2::SensorType::COLOR) {
                    println!("{:?}", stream);
                    // println!("Cropping: {:?}", stream.get_cropping().ok());
                    // println!("Horizontal FOV: {:?}", stream.get_horizontal_fov().ok());
                    // println!("Vertical FOV: {:?}", stream.get_vertical_fov().ok());
                    // println!("Video Mode: {:?}", stream.get_video_mode().ok());
                    // println!("Max Value: {:?}", stream.get_max_value().ok());
                    // println!("Min Value: {:?}", stream.get_min_value().ok());
                    // println!("Stride: {:?}", stream.get_stride().ok());
                    // println!("Mirroring: {:?}", stream.get_mirroring().ok());
                    // println!("Number of frames: {:?}", stream.get_number_of_frames().ok());
                    // println!("Auto White Balance: {:?}", stream.get_auto_white_balance().ok());
                    // println!("Auto Exposure: {:?}", stream.get_auto_exposure().ok());
                    // println!("Auto Exposure: {:?}", stream.get_auto_exposure().ok());
                    // println!("Exposure: {:?}", stream.get_exposure().ok());
                    // println!("Gain: {:?}", stream.get_gain().ok());
                    println!("Starting stream: {:?}", stream.start());
                    {
                        let stream_reader = stream.reader();
                        for _ in 0..5 {
                            let frame = stream_reader.read();
                            println!("Got frame: {:?}", frame.video_mode());
                            // println!("{:?}", frame);
                        }
                    }
                    stream.stop();
                    // println!("{:?}", stream.set_video_mode(OniVideoMode { pixelFormat: OniPixelFormat::ONI_PIXEL_FORMAT_RGB888, resolutionX: 320, resolutionY: 240, fps: 60 }));
                    println!("{:?}", stream.set_video_mode(openni2::VideoMode { pixel_format: openni2::PixelFormat::RGB888, resolution_x: 1280, resolution_y: 960, fps: 30 }));
                    println!("Starting stream: {:?}", stream.start());
                    {
                        let stream_reader = stream.reader();
                        for _ in 0..5 {
                            let frame = stream_reader.read();
                            println!("Got frame: {:?}", frame.video_mode());
                            // println!("{:?}", frame);
                        }
                    }
                    stream.stop();
                }
            }
        },
        openni2::Status::Error(s) => println!("{}", s),
        e @ _ => println!("{:?}", e),
    }
    println!("back outside...");

    openni2::shutdown();
}
