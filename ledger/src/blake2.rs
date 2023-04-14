use blake2::{digest::consts::U32, Blake2b};

pub type Blake256 = Blake2b<U32>;