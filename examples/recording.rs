extern crate openni2;

use openni2::{
    Device,
    SensorType,
    Status,
    Recorder,
    DepthPixel1MM,
};

fn main() -> Result<(), Status> {
    openni2::init()?;

    let device = Device::open_default()?;
    let stream = device.create_stream::<DepthPixel1MM>(SensorType::DEPTH)?;
    let recorder = Recorder::create("./examples/bees/hithere/recording.oni")?;
    recorder.attach_stream(&stream, true)?; // boolean argument: is lossy recording permitted?
    recorder.start()?;
    stream.start()?;

    for _ in 0..5 {
        stream.read_frame()?;
    }

    recorder.stop();
    Ok(())
}
