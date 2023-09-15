#[cfg(feature = "capture")]
use std::net::IpAddr;

#[cfg(feature = "capture")]
use pnet_packet::Packet as _;

#[derive(Clone, Debug, PartialEq)]
pub enum PacketParseError {
    NoNetworkHeader,
    UnsupportedTransport,
    NoTransportHeader,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PacketDirection {
    ToServer,
    FromServer,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PacketProtocol {
    Tcp,
    Udp,
}

#[cfg(feature = "capture")]
impl TryFrom<pnet_packet::ip::IpNextHeaderProtocol> for PacketProtocol {
    type Error = PacketParseError;
    fn try_from(value: pnet_packet::ip::IpNextHeaderProtocol) -> Result<Self, Self::Error> {
        match value {
            pnet_packet::ip::IpNextHeaderProtocols::Tcp => Ok(PacketProtocol::Tcp),
            pnet_packet::ip::IpNextHeaderProtocols::Udp => Ok(PacketProtocol::Udp),
            _ => Err(PacketParseError::UnsupportedTransport),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Packet {
    pub direction: PacketDirection,
    pub protocol: PacketProtocol,
    pub src_port: u16,
    pub dst_port: u16,
    pub data: Vec<u8>,
}

#[cfg(feature = "capture")]
impl Packet {
    pub fn try_parse(
        data: &[u8],
        client_addresses: &[pcap::Address],
    ) -> Result<Packet, PacketParseError> {
        let is_local_address =
            |other: IpAddr| client_addresses.iter().any(|local| local.addr.eq(&other));
        let (direction, protocol, ip_packet) =
            if let Some(ipv4) = pnet_packet::ipv4::Ipv4Packet::new(data) {
                let protocol = ipv4.get_next_level_protocol().try_into()?;
                (
                    if is_local_address(IpAddr::V4(ipv4.get_source())) {
                        PacketDirection::ToServer
                    } else {
                        PacketDirection::FromServer
                    },
                    protocol,
                    Box::new(ipv4) as Box<dyn pnet_packet::Packet>,
                )
            } else if let Some(ipv6) = pnet_packet::ipv6::Ipv6Packet::new(data) {
                let protocol = ipv6.get_next_header().try_into()?;
                (
                    if is_local_address(IpAddr::V6(ipv6.get_source())) {
                        PacketDirection::ToServer
                    } else {
                        PacketDirection::FromServer
                    },
                    protocol,
                    Box::new(ipv6) as Box<dyn pnet_packet::Packet>,
                )
            } else {
                return Err(PacketParseError::NoNetworkHeader);
            };

        let (src_port, dst_port, remaining_data) = match &protocol {
            PacketProtocol::Tcp => {
                let tcp = pnet_packet::tcp::TcpPacket::new(ip_packet.payload())
                    .ok_or(PacketParseError::NoTransportHeader)?;
                (
                    tcp.get_source(),
                    tcp.get_destination(),
                    Vec::from(tcp.payload()),
                )
            }
            PacketProtocol::Udp => {
                let udp = pnet_packet::udp::UdpPacket::new(ip_packet.payload())
                    .ok_or(PacketParseError::NoTransportHeader)?;
                (
                    udp.get_source(),
                    udp.get_destination(),
                    Vec::from(udp.payload()),
                )
            }
        };

        Ok(Packet {
            direction,
            protocol,
            src_port,
            dst_port,
            data: remaining_data,
        })
    }
}
