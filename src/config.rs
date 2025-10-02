use chrono::{DateTime, Utc};

const DATETIME_FORMAT : &str = "%Y-%m-%dT%H:%M:%S%.f%:z";
pub struct Config {
  contact_start_time: Option<DateTime<Utc>>,
  contact_end_time: Option<DateTime<Utc>>, 
  spad_a_channel_id: u32,
  spad_b_channel_id: u32, 
  basis_bit_channel_id: u32
}

impl Config {
  pub fn new(spad_a_channel_id:u32, spad_b_channel_id:u32, basis_bit_channel_id: u32) ->  Self{
    Self {contact_start_time: None, contact_end_time: None, spad_a_channel_id, spad_b_channel_id, basis_bit_channel_id}
  }


  pub fn set_contact_start_time(&mut self, contact_start_time: String) {
    self.contact_start_time =  Some(
      DateTime::parse_from_str(contact_start_time.as_str(), DATETIME_FORMAT).expect("Invalid string format provided to contact_end_time").with_timezone(&Utc)
    );
   }

  pub fn set_contact_end_time(&mut self, contact_end_time: String) {
    self.contact_end_time = Some(
      DateTime::parse_from_str(contact_end_time.as_str(), DATETIME_FORMAT).expect("Invalid string format provided to contact_end_time").with_timezone(&Utc)
    );
  }

  pub fn get_contact_start_time(&self) -> Option<DateTime<Utc>>{
    match self.contact_start_time{
      Some(time) => {
        return Some(time)
      }
      None => {
        return None;
      }

    }
  }

  pub fn get_contact_end_time(&self) -> Option<DateTime<Utc>>{
    match self.contact_end_time{
      Some(time) => {
        return Some(time);
      }
      None => {
        return None;
      }
    }
  }

  pub fn get_spad_a_channel_id(&self) -> u32{
    self.spad_a_channel_id
  }

  pub fn get_spad_b_channel_id(&self) -> u32{
    self.spad_b_channel_id
  }

  pub fn get_basis_bit_channel_id(&self) -> u32{
    self.basis_bit_channel_id
  }
}