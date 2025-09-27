mod basis_stream_generator;
pub mod events;
pub mod basis;
pub struct SinglePhotonDetectionStream {
  channel_id: u32,
  detection_event_stream: Vec<events::PhotonDetectionEvent>
}

impl SinglePhotonDetectionStream {

  pub fn new(channel_id: u32, detection_event_stream: Vec<events::PhotonDetectionEvent>) -> Self {
    Self {
      channel_id,
      detection_event_stream
    }
  }

  pub fn get_detection_event_stream(&mut self) -> &mut Vec<events::PhotonDetectionEvent> {
    &mut self.detection_event_stream
  }

  pub fn print_detection_event_stream(&self) {
    println!("{:#?}", self.detection_event_stream);
  }
}

pub struct BasisBitDetectionStream {
  channel_id: u32, 
  detection_event_stream: Vec<events::BasisBitDetectionEvent>
}

impl BasisBitDetectionStream {

  pub fn new(channel_id: u32, detection_event_stream: Vec<events::BasisBitDetectionEvent>) -> Self {
    Self {
      channel_id,
      detection_event_stream
    }
  }

  pub fn get_detection_event_stream(&mut self) -> &mut Vec<events::BasisBitDetectionEvent> {
    &mut self.detection_event_stream
  }

  pub fn print_detection_event_stream(&self) {
    println!("{:#?}", self.detection_event_stream);
  }
}