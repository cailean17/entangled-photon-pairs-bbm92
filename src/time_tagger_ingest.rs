use crate::streams::basis::Basis;
use crate::config::Config;
use crate::streams::events::{BasisBitDetectionEvent, PhotonDetectionEvent};
use std::io::{self, Read};
use std::process::{Command, Stdio};
use chrono::prelude::*;

pub struct TimeTaggerIngest<'a> {
    config: &'a Config,
}

impl<'a> TimeTaggerIngest<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
    pub fn read(
        &self,
        spad_a_detection_stream: &mut Vec<PhotonDetectionEvent>,
        spad_b_detection_stream: &mut Vec<PhotonDetectionEvent>,
        basis_bit_detection_stream: &mut Vec<BasisBitDetectionEvent>,
    ) -> io::Result<()> {
        // println!("SPAD A: {}, SPAD B: {}, SPAD C: {}", self.config.get_spad_a_channel_id(), self.config.get_spad_b_channel_id(), self.config.get_basis_bit_channel_id());
        // Spawn Time Tagger Ingest C++ Exec
        let mut child = Command::new("./time_tagger_ingest.exe")
            .arg(self.config.get_spad_a_channel_id().to_string())
            .arg(self.config.get_spad_b_channel_id().to_string())
            .arg(self.config.get_basis_bit_channel_id().to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start Time Tagger Ingest C++");

        // Read Pipe output
        let mut stdout = child.stdout.take().expect("Failed to open stdout");
        let header_bytes : usize = 4;
        let mut bytes : u64 = 0;
        loop {
            if Utc::now() < self.config.get_contact_start_time().expect("Entered measurement control loop with an Empty Contact Start Time") {
              continue;
            }
            if Utc::now() > self.config.get_contact_end_time().expect("Entered measurement control loop with an Empty Contact End Time"){
                println!("End Time Reached, stopping measurement!");
                child.kill();
                break; 
            }
            println!("In Measurement Phase");
            let mut header_buffer = vec![0u8; header_bytes];
            match stdout.read_exact(&mut header_buffer){
              Ok(()) => {

              },
              Err(e) => {
                println!("COULD NOT WRITE INTO HEADER BUFFER {}", e);
                break;
              }
            }
            if header_buffer.len() < 4 {
                panic!("Not enough data for event count");
            }
            // Pipe Buffer block format
            // 00 00 00 00 (num events that follow) 
            // [00 00 00 00] [00 00 00 00 00 00 00 00] (ch, time__ps)
            // ..........................................

            // Header Marker, 4 bytes, number of events that follows
            let num_events = u32::from_le_bytes(header_buffer[0..4].try_into().unwrap());
            bytes += (header_bytes as u64 + (num_events*12) as u64);
            let mut buffer = vec![0u8; (num_events * 12) as usize];
            match stdout.read_exact(&mut buffer) {
              Ok(()) => {

              },
              Err(e) => {
                println!("COULD NOT READ INTO EVENT BUFFER: {}", e);
                break;
              }
            }
            // Each event = 12 bytes (4 bytes ch + 8 bytes time_ps)
            for i in 0..num_events as usize {
                let start = i * 12;
                let end = start + 12;
                let chunk = &buffer[start..end];

                let ch = i32::from_le_bytes(chunk[0..4].try_into().unwrap());
                let time_ps = u64::from_le_bytes(chunk[4..12].try_into().unwrap());
                match ch {
                  x if x.abs() as u32 == self.config.get_basis_bit_channel_id() => {
                    basis_bit_detection_stream.push(BasisBitDetectionEvent {
                        time_stamp: time_ps,
                        seq: i,
                        basis: match x {
                          // If the channel number is negative then this event refers to a negative edge event (0 bit)
                          y if y < 0 => {
                            Basis::from_bit(0).unwrap()
                          }
                          // If the channel number is positive then this event refers to a positive edge event (1 bit)
                          y if y > 0 => {
                            Basis::from_bit(1).unwrap()
                          }

                          // default to 0 bit
                          _ => Basis::HorizontalVertical
                        }
                    });
                  }

                  // For SPAD Detections, only take events that come from the positive channels (rising edge detections)
                  x if x >= 0 && x.abs() as u32 == self.config.get_spad_a_channel_id() => {
                    spad_a_detection_stream.push(PhotonDetectionEvent {
                        time_stamp: time_ps,
                        seq: i,
                        ch: ch,
                        basis: None
                    });
                  }
                  x if x >= 0 && x.abs() as u32 == self.config.get_spad_b_channel_id() => {
                    spad_b_detection_stream.push(PhotonDetectionEvent {
                        time_stamp: time_ps,
                        seq: i,
                        ch: ch,
                        basis: None
                    });
                  }

                  _ => println!("Encountered channel id not present in customer config: {}", ch)
                }
            }

            // for (i, event) in spad_a_detection_stream.iter().enumerate() {
            //     println!(
            //         "Event {}: ch={} time_ps={}",
            //         event.seq, event.ch, event.time_stamp
            //     );
            // }
        }

        println!("Bytes Processed: {}", bytes);
        Ok(())
    }
}
