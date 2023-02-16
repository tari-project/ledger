use nanos_sdk::io::Reply;
#[derive(Debug)]
pub enum Error {
    IncorrectByteLength,
    InvalidChallenge,
    ConversionError,
    DecryptFailed,
}

impl Into<Reply> for Error {
    fn into(self) -> Reply {
        match self {
            Error::IncorrectByteLength => Reply(0x69f0_u16),
            Error::InvalidChallenge => Reply(0x9210_u16),
            Error::ConversionError => Reply(0x6a88_u16),
            Error::DecryptFailed => Reply(0x9d60_u16),
        }
    }
}
