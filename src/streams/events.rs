use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct PhotonDetectionEvent{
  pub time_stamp : u64,
  pub seq: usize,
  pub ch: i32,
  pub basis: Option<super::basis::Basis>
}

#[derive(Debug)]
pub struct BasisBitDetectionEvent{
  pub time_stamp: u64, 
  pub seq: usize, 
  pub basis: super::basis::Basis
}