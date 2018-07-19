extern crate openni2;

use openni2::{
    Device,
    SensorType,
    Status,
    Recorder,
    OniDepthPixel,
};

fn main() -> Result<(), Status> {
    openni2::init()?;

    let device = Device::open_default()?;
    let stream = device.create_stream::<OniDepthPixel>(SensorType::DEPTH)?;
    let recorder = Recorder::create("./examples/recording.oni")?;
    recorder.attach_stream(&stream, true)?;
    recorder.start()?;
    stream.start();

    for _ in 0..5 {
        stream.read_frame()?;
    }

    recorder.stop();
    Ok(())
}
