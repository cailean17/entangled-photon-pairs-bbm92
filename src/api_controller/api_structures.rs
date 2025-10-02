use serde::{Serialize,Deserialize};
#[derive(Deserialize, Debug)]
pub struct ControlPlaneStartConnectionResponseBody{
  pub websocket_client_endpoint: String
}

#[derive(Deserialize, Debug)]
pub struct ControlPlaneStartConnectionResponse {
  pub status_code: u32,
  pub body : ControlPlaneStartConnectionResponseBody
}

#[derive(Deserialize, Debug)]
pub struct TelemetryWebSocketContactWindowResponse {
  pub measurement_start_time: String,
  pub measurement_end_time: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelemetryWebSocketMeasurementPresignedUrlResponse {
  pub presigned_url: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TelemetryWebSocketRequestMessage {
  pub user_action: String, 
}
