mod config;
mod time_tagger_ingest;
mod streams;
mod processing; 
mod api_controller; 

use core::panic;

use chrono::{DateTime, Utc};

use crate::api_controller::api_structures;
use crate::config::Config;
use crate::processing::{CorrelateDetectionEventwithActiveBasis, S3Writer};
use crate::time_tagger_ingest::TimeTaggerIngest;
use crate::streams::{BasisBitDetectionStream, SinglePhotonDetectionStream};


#[tokio::main]
async fn main() {
    let CONTROL_PLANE_API : String = String::from("https://5z92ubz4z6.execute-api.us-east-1.amazonaws.com/preproduction");

    let mut conf: Config = Config::new(0,1, 2);
    conf.set_contact_end_time(0);
    conf.set_contact_start_time(0);

    let mut spad_a_stream: SinglePhotonDetectionStream =
        SinglePhotonDetectionStream::new(conf.get_spad_a_channel_id(), Vec::new());

    let mut spad_b_stream: SinglePhotonDetectionStream =
        SinglePhotonDetectionStream::new(conf.get_spad_b_channel_id(), Vec::new());

    let mut basis_bit_stream: BasisBitDetectionStream =
       BasisBitDetectionStream::new(conf.get_basis_bit_channel_id(), Vec::new());
      
    let mut api_controller : api_controller::ApiController = api_controller::ApiController::new(CONTROL_PLANE_API);

    match api_controller.start_control_plane_connection(&conf).await {
      Ok(_value) => {
      },
      Err(e) => {
        println!("Error Starting New BBM92 Connection {}", e);
      }
    };

    match api_controller.start_web_socket_connection().await {
      Ok(_value) => {
        println!("Sucessfully subscribed to BBM92 telemetry websocket");
      },
      Err(e) => {
        println!("Error subscribing to BBM92 telemetry websocket: {}", e);
      }
    }

    loop {
        match api_controller.read_web_socket().await {
          Ok(response) => {
            match response {
                Some(message ) => {
                  println!("SERVER MESSAGE: {:?}", message);
                  let json_str = match message {
                    reqwest_websocket::Message::Text(s) => s,
                    _ => panic!("Expected Message to be of Variant Text")
                  };
                  let contact_window_response : api_structures::TelemetryWebSocketContactWindowResponse= serde_json::from_str(&json_str).expect("Unable to decode json response when reading from BBM92 Telemetry Web Socket");
                  
                  conf.set_contact_start_time(contact_window_response.measurement_start_time.parse::<DateTime<Utc>>().expect("FAILED TO PARSE ISOSTRING").timestamp().clone());
                  conf.set_contact_end_time(contact_window_response.measurement_end_time.parse::<DateTime<Utc>>().expect("FAILED TO PARSE ISOSTRING").timestamp().clone());

                  break;
                },
                None => {
                  continue;
                }
            }
          },
          Err(e) => {
            println!("Error reading from BBM92 telemetry websocket: {}", e);
            break;
          }
        }
    }

    let z: TimeTaggerIngest = TimeTaggerIngest::new(&conf);
    z.read(
        spad_a_stream.get_detection_event_stream(),
        spad_b_stream.get_detection_event_stream(),
        basis_bit_stream.get_detection_event_stream()
    ).expect("Error: ");

    let mut correlator: CorrelateDetectionEventwithActiveBasis = CorrelateDetectionEventwithActiveBasis::new(spad_a_stream.get_detection_event_stream(), spad_b_stream.get_detection_event_stream(), basis_bit_stream.get_detection_event_stream());

    correlator.conduct_correlation();

    let s3_writer : S3Writer = S3Writer::new("https://entangled-bbm92-correlation.s3.us-east-1.amazonaws.com/main.rs?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Content-Sha256=UNSIGNED-PAYLOAD&X-Amz-Credential=AKIA2X3C3XMRW5JEYR4N%2F20250902%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20250902T032628Z&X-Amz-Expires=900&X-Amz-Signature=347c3a03c11708d857d28f0b794400d4f67be5f1d4bfd955cd7274511da91552&X-Amz-SignedHeaders=host&x-id=PutObject".into());
    // s3_writer.write_photon_detection_stream_to_s3_presigned_url(spad_a_stream.get_detection_event_stream()).await;

    // println!("BASIS STREAM:");
    // basis_bit_stream.print_detection_event_stream();
    // println!("SPAD A Stream:");
    // spad_a_stream.print_detection_event_stream();
    // println!("SPAD Y Stream:");
    // spad_b_stream.print_detection_event_stream();
}
