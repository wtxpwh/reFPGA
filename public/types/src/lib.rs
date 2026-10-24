use bincode::{Decode, Encode};

pub mod bimap;
pub mod bitrect;
pub mod bitvec;
pub mod bscan;
pub mod cpld;
pub mod db;
pub mod speed;
pub mod units;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum Polarity {
    P,
    N,
}

impl std::fmt::Display for Polarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Polarity::P => write!(f, "P"),
            Polarity::N => write!(f, "N"),
        }
    }
}

impl std::ops::Not for Polarity {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Polarity::P => Polarity::N,
            Polarity::N => Polarity::P,
        }
    }
}
