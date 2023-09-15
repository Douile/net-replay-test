use std::cell::OnceCell;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::{Arc, Barrier};

use crate::packet::PacketDirection;
use crate::QueryReplay;

pub type ReadyBarrier = Arc<Barrier>;

pub fn udp_server(
    address: SocketAddr,
    query_replay: QueryReplay,
    ready: ReadyBarrier,
) -> std::io::Result<()> {
    let mut packet_pos = 0;
    let packet_count = query_replay.packets.len();
    let mut buf = vec![0u8; query_replay.server.packet_size];
    let client_addr = OnceCell::new();

    let listener = std::net::UdpSocket::bind(&address)?;

    ready.wait();

    while packet_pos < packet_count {
        let packet = &query_replay.packets[packet_pos];

        match packet.direction {
            PacketDirection::ToServer => {
                let (size, address) = listener.recv_from(&mut buf)?;
                if client_addr.get().is_none() {
                    client_addr.set(address).unwrap();
                }
                // TODO: Compare data
                packet_pos += 1;
            }
            PacketDirection::FromServer => {
                let client_addr = client_addr
                    .get()
                    .expect("Must receive a packet before sending");
                let _ = listener.send_to(&packet.data[..], client_addr)?;
                packet_pos += 1;
            }
        }
    }

    Ok(())
}

pub fn tcp_server(
    address: SocketAddr,
    query_replay: QueryReplay,
    ready: ReadyBarrier,
) -> std::io::Result<()> {
    let mut packet_pos = 0;
    let packet_count = query_replay.packets.len();
    let mut buf = vec![0u8; query_replay.server.packet_size];

    let listener = std::net::TcpListener::bind(address)?;

    ready.wait();

    let (mut stream, client_addr) = listener.accept()?;

    while packet_pos < packet_count {
        let packet = &query_replay.packets[packet_pos];
        match packet.direction {
            PacketDirection::ToServer => {
                let size = stream.read(&mut buf)?;
                // TODO: Compare data
                packet_pos += 1;
            }
            PacketDirection::FromServer => {
                let _ = stream.write(&packet.data[..])?;
                packet_pos += 1;
            }
        }
    }

    Ok(())
}
