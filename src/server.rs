use std::cell::OnceCell;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Barrier};

use crate::error::EResult;
use crate::packet::{Packet, PacketDirection, PacketProtocol};
use crate::{Error, QueryReplay};

pub type ReadyBarrier = Arc<Barrier>;

pub fn server(address: IpAddr, query_replay: QueryReplay, ready: ReadyBarrier) -> EResult<()> {
    let mut packet_pos = 0;
    let packet_count = query_replay.packets.len();
    let mut buf = vec![0u8; query_replay.server.packet_size];
    let udp_client_addr = OnceCell::new();
    let tcp_stream = OnceCell::new();

    let tcp_listener = std::net::TcpListener::bind(SocketAddr::new(
        address,
        query_replay.server.tcp_port.unwrap_or(60000),
    ))?;
    let udp_listener = std::net::UdpSocket::bind(SocketAddr::new(
        address,
        query_replay.server.udp_port.unwrap_or(60000),
    ))?;

    ready.wait();

    while packet_pos < packet_count {
        let packet = &query_replay.packets[packet_pos];

        let state = match (&packet.direction, &packet.protocol) {
            (PacketDirection::ToServer, PacketProtocol::Tcp) => {
                handle_tcp_receive(&mut buf, &tcp_stream, &tcp_listener)
            }
            (PacketDirection::ToServer, PacketProtocol::Udp) => {
                handle_udp_receive(&mut buf, &udp_client_addr, &udp_listener)
            }
            (PacketDirection::FromServer, PacketProtocol::Tcp) => {
                handle_tcp_send(packet, &tcp_stream)
            }
            (PacketDirection::FromServer, PacketProtocol::Udp) => {
                handle_udp_send(packet, &udp_client_addr, &udp_listener)
            }
        }?;

        if state == HandleState::Complete {
            packet_pos += 1;
        }
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
enum HandleState {
    Incomplete,
    Complete,
}

fn handle_tcp_receive(
    buf: &mut [u8],
    tcp_stream: &OnceCell<TcpStream>,
    tcp_listener: &TcpListener,
) -> EResult<HandleState> {
    let state = if let Some(mut stream) = tcp_stream.get() {
        let _size = stream.read(buf)?;
        // TODO: Compare data
        HandleState::Complete
    } else {
        let (mut stream, _address) = tcp_listener.accept()?;

        let _size = stream.read(buf)?;
        // TODO: Compare data

        tcp_stream.set(stream).unwrap();

        HandleState::Complete
    };

    Ok(state)
}

fn handle_udp_receive(
    buf: &mut [u8],
    udp_client_addr: &OnceCell<SocketAddr>,
    udp_socket: &UdpSocket,
) -> EResult<HandleState> {
    let (_size, client_addr) = udp_socket.recv_from(buf)?;

    if udp_client_addr.get().is_none() {
        udp_client_addr.set(client_addr).unwrap();
    }

    // TODO: Compare data

    Ok(HandleState::Complete)
}

fn handle_tcp_send(packet: &Packet, tcp_stream: &OnceCell<TcpStream>) -> EResult<HandleState> {
    if let Some(mut stream) = tcp_stream.get() {
        let _size = stream.write(&packet.data[..])?;

        // TODO: Check all sent

        Ok(HandleState::Complete)
    } else {
        Err(Error::SendBeforeRecv(packet.protocol.clone()))
    }
}

fn handle_udp_send(
    packet: &Packet,
    udp_client_addr: &OnceCell<SocketAddr>,
    udp_socket: &UdpSocket,
) -> EResult<HandleState> {
    if let Some(client_addr) = udp_client_addr.get() {
        let _size = udp_socket.send_to(&packet.data[..], client_addr)?;

        // TODO: Check all sent

        Ok(HandleState::Complete)
    } else {
        Err(Error::SendBeforeRecv(packet.protocol.clone()))
    }
}
