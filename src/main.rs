mod api_controller;
mod config;
mod processing;
mod streams;
mod time_tagger_ingest;

use core::panic;
use std::env;

use chrono::{DateTime, Utc};

use crate::api_controller::api_structures::{self, TelemetryWebSocketRequestMessage};
use crate::config::Config;
use crate::processing::{CorrelateDetectionEventwithActiveBasis, S3Writer};
use crate::streams::{BasisBitDetectionStream, SinglePhotonDetectionStream};
use crate::time_tagger_ingest::TimeTaggerIngest;

#[tokio::main]
async fn main() {
    let CONTROL_PLANE_API: String =
        String::from("https://5z92ubz4z6.execute-api.us-east-1.amazonaws.com/preproduction");

    let args: Vec<String> = env::args().collect();

    match args.len() {
        6 => {}
        _ => {
            panic!("ERROR: MISSING ARGUMENTS, ABORTING RUN");
        }
    }

    let mut conf: Config = Config::new(
        args[1].parse::<u32>().unwrap(),
        args[2].parse::<u32>().unwrap(),
        args[3].parse::<u32>().unwrap(),
    );

    let mut spad_a_stream: SinglePhotonDetectionStream =
        SinglePhotonDetectionStream::new(conf.get_spad_a_channel_id(), Vec::new());

    let mut spad_b_stream: SinglePhotonDetectionStream =
        SinglePhotonDetectionStream::new(conf.get_spad_b_channel_id(), Vec::new());

    let mut basis_bit_stream: BasisBitDetectionStream =
        BasisBitDetectionStream::new(conf.get_basis_bit_channel_id(), Vec::new());

    let mut api_controller: api_controller::ApiController =
        api_controller::ApiController::new(CONTROL_PLANE_API);

    if (args[4] == "INITIATE") {
        match api_controller.control_plane_start_new_connection().await {
            Ok(_value) => {
                println!("Sucessfully created a new BBM92 Connection");
            }
            Err(e) => {
                println!("Error Starting New BBM92 Connection {}", e);
            }
        };

        match api_controller.web_socket_join_connection(&conf).await {
            Ok(_value) => {
                println!("Sucessfully subscribed to BBM92 Connection's telemetry websocket");
            }
            Err(e) => {
                println!(
                    "Error subscribing to BBM92 Connection's telemetry websocket: {}",
                    e
                );
            }
        }
    } else {
        api_controller.set_web_socket_url(Some(args[5].clone()));
        match api_controller.web_socket_join_connection(&conf).await {
            Ok(_value) => {
                println!("Sucessfully subscribed to BBM92 Connection's telemetry websocket");
            }
            Err(e) => {
                println!(
                    "Error subscribing to BBM92 Connection's telemetry websocket: {}",
                    e
                );
            }
        }
    }

    println!("POLLING FOR WEBSOCKET SERVER MESSAGE...");
    match api_controller.web_socket_read().await {
        Ok(response) => match response {
            Some(message) => {
                println!("SERVER MESSAGE: {:?}", message);
                let json_str = match message {
                    reqwest_websocket::Message::Text(s) => s,
                    _ => panic!("Expected Message to be of Variant Text"),
                };
                let contact_window_response : api_structures::TelemetryWebSocketContactWindowResponse= serde_json::from_str(&json_str).expect("Unable to decode json response when reading from BBM92 Telemetry Web Socket");

                conf.set_contact_start_time(contact_window_response.measurement_start_time);
                conf.set_contact_end_time(contact_window_response.measurement_end_time);
            }
            None => {
                panic!("WEB SOCKET CONNECTION CLOSED BY SERVER - ABORTING PROTOCOL");
            }
        },
        Err(e) => {
            panic!("Error reading from BBM92 telemetry websocket: {}", e);
        }
    }

    let z: TimeTaggerIngest = TimeTaggerIngest::new(&conf);
    z.read(
        spad_a_stream.get_detection_event_stream(),
        spad_b_stream.get_detection_event_stream(),
        basis_bit_stream.get_detection_event_stream(),
    )
    .expect("Error: ");

    let mut correlator: CorrelateDetectionEventwithActiveBasis =
        CorrelateDetectionEventwithActiveBasis::new(
            spad_a_stream.get_detection_event_stream(),
            spad_b_stream.get_detection_event_stream(),
            basis_bit_stream.get_detection_event_stream(),
        );
    correlator.conduct_correlation();

    match api_controller
        .web_socket_send(TelemetryWebSocketRequestMessage {
            user_action: "requestMeasurementPresignedUrl".to_string(),
        })
        .await
    {
        Ok(_message) => {}
        Err(e) => {
            panic!("Error sending message to BBM92 telemetry websocket: {}", e);
        }
    }

    println!("POLLING FOR WEBSOCKET SERVER MESSAGE...");
    match api_controller.web_socket_read().await {
        Ok(response) => match response {
            Some(message) => {
                println!("SERVER MESSAGE: {:?}", message);
                let json_str = match message {
                    reqwest_websocket::Message::Text(s) => s,
                    _ => panic!("Expected Message to be of Variant Text"),
                };
                let contact_window_response : api_structures::TelemetryWebSocketMeasurementPresignedUrlResponse= serde_json::from_str(&json_str).expect("Unable to decode json response when reading from BBM92 Telemetry Web Socket");
                
                let s3_writer : S3Writer = S3Writer::new(contact_window_response.presigned_url.clone());
                s3_writer.write_photon_detection_stream_to_s3_presigned_url(spad_a_stream.get_detection_event_stream()).await;
            }
            None => {
                panic!("WEB SOCKET CONNECTION CLOSED BY SERVER - ABORTING PROTOCOL");
            }
        },
        Err(e) => {
            panic!("Error reading from BBM92 telemetry websocket: {}", e);
        }
    }
}
