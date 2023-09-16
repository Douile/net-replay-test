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
    pub tcp_port: Option<u16>,
    pub udp_port: Option<u16>,
    pub packet_size: usize,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ServerOptionsError {
    NoPort,
    AmbiguousPort(HashSet<u16>),
}

impl TryFrom<&[Packet]> for ServerOptions {
    type Error = ServerOptionsError;
    fn try_from(packets: &[Packet]) -> Result<Self, Self::Error> {
        let mut tcp_ports = HashSet::new();
        let mut udp_ports = HashSet::new();
        let mut max_packet_size = usize::MIN;

        for packet in packets {
            let port = match packet.direction {
                PacketDirection::ToServer => packet.dst_port,
                PacketDirection::FromServer => packet.src_port,
            };
            match packet.protocol {
                PacketProtocol::Tcp => tcp_ports.insert(port),
                PacketProtocol::Udp => udp_ports.insert(port),
            };
            max_packet_size = max_packet_size.max(packet.data.len());
        }

        if tcp_ports.len() > 1 {
            return Err(ServerOptionsError::AmbiguousPort(tcp_ports));
        }

        if udp_ports.len() > 1 {
            return Err(ServerOptionsError::AmbiguousPort(udp_ports));
        }

        let tcp_port = tcp_ports.into_iter().next();
        let udp_port = udp_ports.into_iter().next();

        if tcp_port.is_none() && udp_port.is_none() {
            Err(ServerOptionsError::NoPort)
        } else {
            Ok(ServerOptions {
                tcp_port,
                udp_port,
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
    pub replay_version: u32,
}
