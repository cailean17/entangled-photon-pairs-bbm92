use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Basis{
  DiagonalAntiDiagonal,
  HorizontalVertical
}

impl Basis {
  pub fn from_bit(bit: u8) -> Option<Self> {
    match bit {
      0 => Some(Basis::HorizontalVertical),
      1 => Some(Basis::DiagonalAntiDiagonal),
      _ => None
    }
  }
}