mod basis;
mod basis_stream_generator;
mod config;
mod events;
mod time_tagger_ingest;
mod streams;
mod processing; 

use crate::config::Config;
use crate::processing::{CorrelateDetectionEventwithActiveBasis, S3Writer};
use crate::time_tagger_ingest::TimeTaggerIngest;
use crate::streams::{BasisBitDetectionStream, SinglePhotonDetectionStream};

#[tokio::main]
async fn main() {
    let conf: Config = Config::new(0, 0, 0,1,2);

    let mut spad_a_stream: SinglePhotonDetectionStream =
        SinglePhotonDetectionStream::new(conf.get_spad_a_channel_id(), Vec::new());

    let mut spad_b_stream: SinglePhotonDetectionStream =
        SinglePhotonDetectionStream::new(conf.get_spad_b_channel_id(), Vec::new());

    let mut basis_bit_stream: BasisBitDetectionStream =
       BasisBitDetectionStream::new(conf.get_basis_bit_channel_id(), Vec::new());

    let z: TimeTaggerIngest = TimeTaggerIngest::new(&conf);
    z.read(
        spad_a_stream.get_detection_event_stream(),
        spad_b_stream.get_detection_event_stream(),
        basis_bit_stream.get_detection_event_stream()
    ).expect("Error: ");

    let mut correlator: CorrelateDetectionEventwithActiveBasis = CorrelateDetectionEventwithActiveBasis::new(spad_a_stream.get_detection_event_stream(), spad_b_stream.get_detection_event_stream(), basis_bit_stream.get_detection_event_stream());

    correlator.conduct_correlation();

    let s3_writer : S3Writer = S3Writer::new("https://entangled-bbm92-correlation.s3.us-east-1.amazonaws.com/main.rs?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Content-Sha256=UNSIGNED-PAYLOAD&X-Amz-Credential=AKIA2X3C3XMRW5JEYR4N%2F20250902%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20250902T032628Z&X-Amz-Expires=900&X-Amz-Signature=347c3a03c11708d857d28f0b794400d4f67be5f1d4bfd955cd7274511da91552&X-Amz-SignedHeaders=host&x-id=PutObject".into());
    s3_writer.write_photon_detection_stream_to_s3_presigned_url(spad_a_stream.get_detection_event_stream()).await;

    // println!("BASIS STREAM:");
    // basis_bit_stream.print_detection_event_stream();
    // println!("SPAD A Stream:");
    // spad_a_stream.print_detection_event_stream();
    // println!("SPAD Y Stream:");
    // spad_b_stream.print_detection_event_stream();
}
