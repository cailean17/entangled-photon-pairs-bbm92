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

    // let s3_writer : S3Writer = S3Writer::new("https://entangled-bbm92-correlation.s3.us-east-1.amazonaws.com/test-measurement-1.json.txt?response-content-disposition=inline&X-Amz-Content-Sha256=UNSIGNED-PAYLOAD&X-Amz-Security-Token=IQoJb3JpZ2luX2VjEIf%2F%2F%2F%2F%2F%2F%2F%2F%2F%2FwEaCXVzLWVhc3QtMSJHMEUCIQDqW0EE2ovRTI0o3o%2FI2cKKK2Vgv3PKqxKrYtlfTW%2BpRwIgZj5o2%2FxW0kRae%2Bkm2gVp6NO00oo5GOVrmHaMUCbpX%2BsqtQMIIBAAGgw3Mzg0MDQ4NDIyNzUiDLIGKMI5meAW25Y1PyqSA1XJ38NcZPuvtYQkarBfdrYhXjFc3Gz0X8rqBSqVheaChGROITdNWBRinJt3gO2a61oln5d9NCuNlctEzMHGlOa3MY%2FD2%2FRA4CEUAg3hl7ILgyjlHjvy%2F0HK5B3CC33IoPmy8oYwn2FhsQ7ZyH8pMSeyqG2YsRobVUEBBR2a6ux600X9jcbwHGRnUz8YwGDnfZxUulLjitJcI3hMvVdAMVfcRl194q%2Bn%2FQczkj9UxTmMY77wOgGgiONWtavLs2hbTHf4xPcNkSs1xtgnHkqv3b23pdKJqGOxgt8eXq85HcPV2KHmRdB5muXVKIZj4Ebyo%2Fo6nB4KTNwjuZVRDN%2BG6bARYLZSY0ZIfLN8bWZur%2FJWBkmBtHS1lTTjYNeP8DtCbYBReoP%2FwzWvOP6xr4PeSkZe%2FOcFixv2jB0ozTdaj9Xe5oG%2FvVPZJM%2FVupBOlytPJME7Xg3QNJ%2BbKh6lQ5ruNQU2sxlQskonxPESJSUmiRgiFbxoqhv8JXzitc1Vs69yTxcUO3pUQnFjnhWrfzBLPGEfhDCq5vXGBjreAgomQdaWdbLcBCdzX%2FZ0vJqLMSsvuDnuwStyM4CH%2FGOKfaZ%2B9zf96ga%2F5QHFyFDWXMNLnCXnROfqrQsXXwedjTe2BoSZy6o%2B%2F0vvuduSNhuXl1EJrEk5Tuv6y95BNqTCb1DT0tCZ2iVT%2BwBlEeXFXGJVZtnQz4lzw5L5AswOR3NDSZF4Ar2zMMlZcNujXfmIHlxQJTCWkBc6LesaVMTI8uZ7qF24fOGGuIAYVJIKN%2BXen7P%2BEuTQRjcSXzZeU0I2moWivSFNEymZAl0Pq6OnU5bvPyRSyKqr3WiqQSQmeKc8S5aHs95yfPtv%2BFKcbG1IUH0DLWDt6Ahy8ixzhCfWqF3kDp2ME4nMT8zoj4Y584HNuZhokDL50aMCGYqZE%2BXk4CMnVMxWO3Al%2FJN%2BqHSRcSydwZogik1JuGpn5kyLIV%2Fov4VZhMu3npoqbaEYQdewu8c0wYwnOh6Q5QDfcAt4&X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=ASIA2X3C3XMR3ZLCKXLR%2F20251001%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20251001T225511Z&X-Amz-Expires=1200&X-Amz-SignedHeaders=host&X-Amz-Signature=2c3223f91f42547440d6fc773b40991d7099571da57180f6e0b332aa1938a2d8".into());
    // s3_writer.write_photon_detection_stream_to_s3_presigned_url(spad_a_stream.get_detection_event_stream()).await;
}
