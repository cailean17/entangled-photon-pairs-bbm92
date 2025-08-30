use crate::events::{BasisBitDetectionEvent, PhotonDetectionEvent};

pub struct SinglePhotonDetectionStream {
  channel_id: u32,
  detection_event_stream: Vec<PhotonDetectionEvent>
}

impl SinglePhotonDetectionStream {

  pub fn new(channel_id: u32, detection_event_stream: Vec<PhotonDetectionEvent>) -> Self {
    Self {
      channel_id,
      detection_event_stream
    }
  }

  pub fn get_detection_event_stream(&mut self) -> &mut Vec<PhotonDetectionEvent> {
    &mut self.detection_event_stream
  }

  pub fn print_detection_event_stream(&self) {
    println!("{:#?}", self.detection_event_stream);
  }
}

pub struct BasisBitDetectionStream {
  channel_id: u32, 
  detection_event_stream: Vec<BasisBitDetectionEvent>
}

impl BasisBitDetectionStream {

  pub fn new(channel_id: u32, detection_event_stream: Vec<BasisBitDetectionEvent>) -> Self {
    Self {
      channel_id,
      detection_event_stream
    }
  }

  pub fn get_detection_event_stream(&mut self) -> &mut Vec<BasisBitDetectionEvent> {
    &mut self.detection_event_stream
  }

  pub fn print_detection_event_stream(&self) {
    println!("{:#?}", self.detection_event_stream);
  }
}