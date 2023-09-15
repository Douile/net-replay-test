use std::collections::HashSet;

use crate::packet::{Packet, PacketDirection, PacketProtocol};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct QueryOptions {
    pub address: String,
    pub port: Option<u16>,
    pub game: String,
}

#[cfg(feature = "cli")]
impl QueryOptions {
    pub fn as_file_name(&self) -> String {
        let date = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        format!("replay-{}-{}-{}.json", date, self.game, self.address)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ServerOptions {
    pub protocol: PacketProtocol,
    pub port: u16,
    pub packet_size: usize,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ServerOptionsError {
    AmbiguousProtocol,
    AmbiguousPort(HashSet<u16>),
}

impl TryFrom<&[Packet]> for ServerOptions {
    type Error = ServerOptionsError;
    fn try_from(packets: &[Packet]) -> Result<Self, Self::Error> {
        let mut seen_protocols = HashSet::new();
        let mut seen_ports = HashSet::new();
        let mut max_packet_size = usize::MIN;

        for packet in packets {
            seen_protocols.insert(packet.protocol.clone());
            match packet.direction {
                PacketDirection::ToServer => {
                    seen_ports.insert(packet.dst_port);
                }
                PacketDirection::FromServer => {
                    seen_ports.insert(packet.src_port);
                }
            }
            max_packet_size = max_packet_size.max(packet.data.len());
        }

        if seen_protocols.len() != 1 {
            Err(ServerOptionsError::AmbiguousProtocol)
        } else if seen_ports.len() != 1 {
            Err(ServerOptionsError::AmbiguousPort(seen_ports))
        } else {
            Ok(ServerOptions {
                protocol: seen_protocols.into_iter().next().unwrap(),
                port: seen_ports.into_iter().next().unwrap(),
                packet_size: max_packet_size,
            })
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct QueryReplay {
    pub query: QueryOptions,
    pub server: ServerOptions,
    pub packets: Vec<Packet>,
    pub value: serde_json::Value,
}
