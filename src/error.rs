use std::fmt::Formatter;

use crate::options::ServerOptionsError;
use crate::packet::{PacketParseError, PacketProtocol};

pub type GenericError = Box<dyn std::error::Error + 'static>;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "capture")]
    Pcap(pcap::Error),
    #[cfg(feature = "capture")]
    NoCaptureDevice,
    IO(std::io::Error),
    Json(serde_json::Error),
    PacketParse(PacketParseError),
    ServerOptions(ServerOptionsError),
    String(String),
    #[cfg(feature = "replay")]
    SendBeforeRecv(PacketProtocol),
    #[cfg(feature = "impl_rs")]
    Rust(gamedig::GDError),
    WrongReplayVersion {
        found: u32,
        required: u32,
    },
    #[cfg(feature = "filter")]
    Filter(crate::packet_filter::FilterError),
    Generic(GenericError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "{:#?}", self)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "capture")]
            Self::NoCaptureDevice => None,
            Self::WrongReplayVersion {
                found: _,
                required: _,
            } => None,
            Self::PacketParse(_) => None,
            Self::ServerOptions(_) => None,
            #[cfg(feature = "replay")]
            Self::SendBeforeRecv(_) => None,
            #[cfg(feature = "filter")]
            Self::Filter(_) => None,
            Self::String(_) => None,

            #[cfg(feature = "capture")]
            Self::Pcap(source) => Some(source),
            Self::IO(source) => Some(source),
            Self::Json(source) => Some(source),
            Self::Generic(source) => Some(source.as_ref()),
            #[cfg(feature = "impl_rs")]
            Self::Rust(source) => Some(source),
        }
    }
}

pub type EResult<T> = Result<T, Error>;

#[cfg(feature = "capture")]
impl From<pcap::Error> for Error {
    fn from(value: pcap::Error) -> Self {
        Error::Pcap(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IO(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value)
    }
}

impl From<PacketParseError> for Error {
    fn from(value: PacketParseError) -> Self {
        Error::PacketParse(value)
    }
}

impl From<ServerOptionsError> for Error {
    fn from(value: ServerOptionsError) -> Self {
        Error::ServerOptions(value)
    }
}

#[cfg(feature = "impl_rs")]
impl From<gamedig::GDError> for Error {
    fn from(value: gamedig::GDError) -> Self {
        Error::Rust(value)
    }
}

#[cfg(feature = "filter")]
impl From<crate::packet_filter::FilterError> for Error {
    fn from(value: crate::packet_filter::FilterError) -> Self {
        Error::Filter(value)
    }
}

impl From<GenericError> for Error {
    fn from(value: GenericError) -> Self {
        Error::Generic(value)
    }
}
