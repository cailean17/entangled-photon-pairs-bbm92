mod basis;
mod basis_stream_generator;
mod config;
mod events;
mod time_tagger_ingest;
mod streams;

use crate::basis_stream_generator::BasisStreamGenerator;
use crate::config::Config;
use crate::time_tagger_ingest::TimeTaggerIngest;
use crate::streams::{BasisBitDetectionStream, SinglePhotonDetectionStream};
fn main() {
    let conf: Config = Config::new(0, 0, 0,1,2);

    let w: BasisStreamGenerator = BasisStreamGenerator::new(None);

    let mut spad_a_stream: SinglePhotonDetectionStream =
        SinglePhotonDetectionStream::new(conf.get_spad_a_channel_id(), Vec::new());

    let mut spad_b_stream: SinglePhotonDetectionStream =
        SinglePhotonDetectionStream::new(conf.get_spad_b_channel_id(), Vec::new());

    let mut basis_bit_stream: BasisBitDetectionStream =
       BasisBitDetectionStream::new(conf.get_basis_bit_channel_id(), Vec::new());

    let z: TimeTaggerIngest = TimeTaggerIngest::new(&conf);
    // y.obtain_available_port_lists();
    // y.generate_basis_blocks();
    z.read(
        spad_a_stream.get_detection_event_stream(),
        spad_b_stream.get_detection_event_stream(),
        basis_bit_stream.get_detection_event_stream()
    ).expect("Error: ");
    println!("SPAD A Stream:");
    spad_a_stream.print_detection_event_stream();
    println!("SPAD Y Stream:");
    spad_b_stream.print_detection_event_stream();
}
