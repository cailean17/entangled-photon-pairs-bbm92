pub struct Config {
  contact_start_time: Option<i64>,
  contact_end_time: Option<i64>, 
  spad_a_channel_id: u32, 
  spad_b_channel_id: u32, 
  basis_bit_channel_id: u32
}

impl Config {
  pub fn new(spad_a_channel_id:u32, spad_b_channel_id:u32, basis_bit_channel_id: u32) ->  Self{
    Self {contact_start_time: None, contact_end_time: None, spad_a_channel_id, spad_b_channel_id, basis_bit_channel_id}
  }


  pub fn set_contact_start_time(&mut self, contact_start_time: i64) {
    self.contact_start_time = Some(contact_start_time);
  }

  pub fn set_contact_end_time(&mut self, contact_end_time: i64) {
    self.contact_end_time = Some(contact_end_time);
  }

  pub fn get_contact_start_time(&self) -> i64{
    match self.contact_start_time{
      Some(time) => {
        return time
      }
      None => {
        return 0
      }

    }
  }

  pub fn get_contact_end_time(&self) -> i64{
    match self.contact_end_time{
      Some(time) => {
        return time
      }
      None => {
        return 0
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