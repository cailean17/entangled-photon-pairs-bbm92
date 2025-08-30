pub struct Config {
  contact_start_time: u64,
  contact_end_time: u64, 
  spad_a_channel_id: u32, 
  spad_b_channel_id: u32, 
  basis_bit_channel_id: u32
}

impl Config {
  pub fn new(contact_start_time:u64, contact_end_time:u64, spad_a_channel_id:u32, spad_b_channel_id:u32, basis_bit_channel_id: u32) ->  Self{
    Self {contact_start_time, contact_end_time, spad_a_channel_id, spad_b_channel_id, basis_bit_channel_id}
  }

  pub fn get_contact_start_time(&self) -> u64{
    self.contact_start_time
  }

  pub fn get_contact_end_time(&self) -> u64{
    self.contact_end_time
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