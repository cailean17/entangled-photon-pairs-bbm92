use crate::basis::Basis;

#[derive(Debug)]
pub struct PhotonDetectionEvent{
  pub time_stamp : u64,
  pub seq: usize,
  pub ch: i32
}

#[derive(Debug)]
pub struct BasisBitDetectionEvent{
  pub time_stamp: u64, 
  pub seq: usize, 
  pub basis: Basis
}