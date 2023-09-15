use crate::options::ServerOptionsError;
use crate::packet::PacketParseError;

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
}

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
