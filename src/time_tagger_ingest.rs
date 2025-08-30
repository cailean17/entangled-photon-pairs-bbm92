use crate::basis::Basis;
use crate::config::Config;
use crate::events::{BasisBitDetectionEvent, PhotonDetectionEvent};
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
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
        // Spawn Time Tagger Ingest C++ Exec
        let mut child = Command::new("../time_tagger_ingest.exe")
            .arg(self.config.get_spad_a_channel_id().to_string())
            .arg(self.config.get_spad_b_channel_id().to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start Time Tagger Ingest C++");

        // Read Pipe output
        let stdout = child.stdout.as_mut().expect("Failed to open stdout");

        let start_time = Instant::now();
        let loop_duration = Duration::from_secs(10);
        let mut bytes: u64 = 0;
        loop {
            if Instant::now().duration_since(start_time) > loop_duration {
                break;
            }
            bytes += (4096 * 12) + 4;
            let mut buffer = vec![0u8; 4 + 4096 * 12];

            stdout.read_exact(&mut buffer)?;

            if buffer.len() < 4 {
                panic!("Not enough data for event count");
            }

            // Pipe Buffer block format
            // 00 00 00 00 (num events that follow) 
            // [00 00 00 00] [00 00 00 00 00 00 00 00] (ch, time__ps)
            // ..........................................

            // Header Marker, 4 bytes, number of events that follows
            let num_events = u32::from_le_bytes(buffer[0..4].try_into().unwrap());

            // Each event = 12 bytes (4 bytes ch + 8 bytes time_ps)
            for i in 0..num_events as usize {
                let start = 4 + i * 12;
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

                  // For SPAD Detections, only take events that come from the positive edge channels
                  x if x > 0 && x.abs() as u32 == self.config.get_spad_a_channel_id() => {
                    spad_a_detection_stream.push(PhotonDetectionEvent {
                        time_stamp: time_ps,
                        seq: i,
                        ch: ch,
                    });
                  }
                  x if x > 0 && x.abs() as u32 == self.config.get_spad_a_channel_id() => {
                    spad_b_detection_stream.push(PhotonDetectionEvent {
                        time_stamp: time_ps,
                        seq: i,
                        ch: ch,
                    });
                  }

                  _ => println!("Encountered channel id not present in customer config: {}", ch)
                }
            }

            // for (i, event) in events.iter().enumerate() {
            //     println!(
            //         "Event {}: ch={} time_ps={}",
            //         event.seq, event.ch, event.time_stamp
            //     );
            // }
        }

        println!("Bytes Processed in 10 Seconds: {}", bytes);
        Ok(())
    }
}
