#[cfg(feature = "replay")]
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[cfg(feature = "capture")]
use pcap::{Capture, Device};

pub mod error;
pub use error::Error;

pub mod implementations;
#[cfg(any(feature = "capture", feature = "replay"))]
use implementations::QueryImplementation;

pub mod packet;
#[cfg(feature = "capture")]
use packet::Packet;

#[cfg(feature = "replay")]
pub mod server;

pub mod options;
pub use options::{QueryOptions, QueryReplay};

#[cfg(feature = "capture")]
fn create_pcap_capture(
    options: &QueryOptions,
    device_name: Option<&str>,
) -> Result<(Vec<pcap::Address>, Capture<pcap::Active>), Error> {
    let device = if let Some(device_name) = device_name {
        let devices = Device::list()?;
        devices
            .into_iter()
            .filter(|d| d.name == device_name)
            .next()
            .ok_or(Error::NoCaptureDevice)?
    } else {
        Device::lookup()?.ok_or(Error::NoCaptureDevice)?
    };
    let addresses = device.addresses.clone();
    println!("Capturing using {:?}", device);
    let mut capture = Capture::from_device(device)?.immediate_mode(true).open()?;

    let filter = format!("host {}", options.address);
    println!("filter: {}", filter);
    capture.filter(&filter, true)?;
    Ok((addresses, capture))
}

#[cfg(feature = "capture")]
pub fn capture(
    implementation: Box<dyn QueryImplementation>,
    options: QueryOptions,
    device_name: Option<&str>,
) -> Result<QueryReplay, Error> {
    let (addresses, mut capture) = create_pcap_capture(&options, device_name)?;

    let mut save_file = capture.savefile("test.pcap")?;

    let value = implementation.query_server(&options)?;

    println!("{:#?}", value);
    println!("{:#?}", capture.stats());

    let mut capture = capture.setnonblock()?;

    let mut packets = Vec::new();
    while let Ok(packet) = capture.next_packet() {
        save_file.write(&packet); // Write to save file as backup

        let pkt = Packet::try_parse(packet.data, &addresses)?;
        packets.push(pkt);
    }

    println!("{:?}", packets);

    let server_options = options::ServerOptions::try_from(&packets[..])?;

    Ok(QueryReplay {
        query: options,
        server: server_options,
        packets,
        value,
    })
}

#[cfg(feature = "replay")]
pub fn replay(
    implementation: Box<dyn QueryImplementation>,
    query_replay: QueryReplay,
) -> Result<(), Error> {
    use std::sync::{Arc, Barrier};

    let address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 50));

    let mut query_options = query_replay.query.clone();

    let barrier = Arc::new(Barrier::new(2));

    let server_barrier = Arc::clone(&barrier);
    let server_thread = std::thread::spawn(move || {
        server::server(address, query_replay, server_barrier).unwrap();
    });

    barrier.wait();

    query_options.address = address.to_string();
    let value = implementation.query_server(&query_options)?;

    println!("Value {:#?}", value);

    let _ = server_thread.join();

    // TODO: Compare values

    Ok(())
}
