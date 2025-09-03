use std::{fs::File, time::UNIX_EPOCH};

use crate::{
    basis::Basis,
    events::{BasisBitDetectionEvent, PhotonDetectionEvent},
};
use std::time::{Duration, Instant, SystemTime};

use rand::Rng;
use reqwest::Client;

pub struct CorrelateDetectionEventwithActiveBasis<'a> {
    spad_a_detection_stream: &'a mut Vec<PhotonDetectionEvent>,
    spad_b_detection_stream: &'a mut Vec<PhotonDetectionEvent>,
    basis_bit_detection_stream: &'a mut Vec<BasisBitDetectionEvent>,
}

impl<'a> CorrelateDetectionEventwithActiveBasis<'a> {
    pub fn new(
        spad_a_detection_stream: &'a mut Vec<PhotonDetectionEvent>,
        spad_b_detection_stream: &'a mut Vec<PhotonDetectionEvent>,
        basis_bit_detection_stream: &'a mut Vec<BasisBitDetectionEvent>,
    ) -> Self {
        Self {
            spad_a_detection_stream,
            spad_b_detection_stream,
            basis_bit_detection_stream,
        }
    }

    pub fn conduct_correlation(&mut self) {
        let mut ptr_spad_a: usize = 0;
        let mut ptr_spad_b: usize = 0;
        let mut ptr_basis: usize = 0;
        let mut current_basis: Basis = Basis::HorizontalVertical;

        while ptr_basis < self.basis_bit_detection_stream.len() {
            let basis_timestamp: &u64 = &self.basis_bit_detection_stream[ptr_basis].time_stamp;
            // println!("PROCESSING BASIS TIMESTAMP: {} WITH VALUE {:?}", basis_timestamp, self.basis_bit_detection_stream[ptr_basis].basis);
            while ptr_spad_a < self.spad_a_detection_stream.len()
                && self.spad_a_detection_stream[ptr_spad_a].time_stamp < *basis_timestamp
            {
                self.spad_a_detection_stream[ptr_spad_a].basis = Some(current_basis);
                ptr_spad_a += 1;
            }
            while ptr_spad_b < self.spad_b_detection_stream.len()
                && self.spad_b_detection_stream[ptr_spad_b].time_stamp < *basis_timestamp
            {
                self.spad_b_detection_stream[ptr_spad_b].basis = Some(current_basis);
                ptr_spad_b += 1;
            }
            current_basis = self.basis_bit_detection_stream[ptr_basis].basis.clone();
            ptr_basis += 1;
        }
        // Trailing Spad Events after last basis bit change
        while ptr_spad_a < self.spad_a_detection_stream.len() {
            self.spad_a_detection_stream[ptr_spad_a].basis = Some(current_basis);
            ptr_spad_a += 1;
        }
        while ptr_spad_b < self.spad_b_detection_stream.len() {
            self.spad_b_detection_stream[ptr_spad_b].basis = Some(current_basis);
            ptr_spad_b += 1;
        }
    }
}

pub struct S3Writer{
  presigned_url_write: String
}

impl S3Writer{
  pub fn new(presigned_url_write: String) -> Self {
    Self {presigned_url_write}
  }

  pub async fn write_photon_detection_stream_to_s3_presigned_url(&self, stream: &Vec<PhotonDetectionEvent>) -> Result<(), Box<dyn std::error::Error>>{
    // let client = reqwest::Client::new();
    let mut rng = rand::thread_rng();
    let filename = format!("test-measurement-{}txt", SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros());
    let mut output = File::create(filename).expect("FAILED TO CREATE FILE");
    serde_json::to_writer(&mut output, &stream[0..100])?;
    // let response = client.put(&self.presigned_url_write).body(json_string).send().await?;

    // println!("RESPONSE: {}", response.status());
    Ok(())
  }
}