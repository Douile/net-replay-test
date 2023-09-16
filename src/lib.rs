#[cfg(feature = "replay")]
use std::net::{IpAddr, Ipv4Addr};

#[cfg(feature = "capture")]
use std::path::Path;

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

pub mod value;

pub const REPLAY_VERSION: u32 = 1;

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

/// Capture a query using the given implementation, device name can be used to specify which
/// network device to capture traffic on. To capture traffic this function requires elevated system
/// privileges.
#[cfg(feature = "capture")]
pub fn capture(
    implementation: Box<dyn QueryImplementation>,
    options: QueryOptions,
    device_name: Option<&str>,
    pcap_file: Option<impl AsRef<Path>>,
) -> Result<QueryReplay, Error> {
    let (addresses, mut capture) = create_pcap_capture(&options, device_name)?;

    let mut save_file = if let Some(pcap_file) = pcap_file {
        Some(capture.savefile(pcap_file)?)
    } else {
        None
    };

    let value = implementation.query_server(&options);

    println!("Packets captured {:#?}", capture.stats());

    let mut capture = capture.setnonblock()?;

    let mut packets = Vec::new();
    while let Ok(packet) = capture.next_packet() {
        if let Some(ref mut save_file) = save_file {
            save_file.write(&packet); // Write to save file as backup
        }

        let pkt = Packet::try_parse(packet.data, &addresses)?;
        packets.push(pkt);
    }

    let value = value?;
    println!("{:#?}", value);
    println!("{:?}", packets);

    let server_options = options::ServerOptions::try_from(&packets[..])?;

    Ok(QueryReplay {
        query: options,
        server: server_options,
        packets,
        value,
        replay_version: REPLAY_VERSION,
    })
}

/// Replay a saved query using a given implementation, return whether the output value (if
/// successful) matches
#[cfg(feature = "replay")]
pub fn replay(
    implementation: Box<dyn QueryImplementation>,
    query_replay: QueryReplay,
) -> Result<bool, Error> {
    use std::sync::{Arc, Barrier};

    if query_replay.replay_version != REPLAY_VERSION {
        return Err(Error::WrongReplayVersion {
            found: query_replay.replay_version,
            required: REPLAY_VERSION,
        });
    }

    let address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 50));

    let mut query_options = query_replay.query.clone();
    let query_value = query_replay.value.clone();

    let barrier = Arc::new(Barrier::new(2));

    let server_barrier = Arc::clone(&barrier);
    let server_thread = std::thread::spawn(move || {
        server::server(address, query_replay, server_barrier).unwrap();
    });

    query_options.address = address.to_string();

    barrier.wait();

    let start_time = std::time::Instant::now();
    let value = implementation.query_server(&query_options)?;
    let duration = std::time::Instant::now() - start_time;

    let _ = server_thread.join();

    let values_match = value == query_value;

    println!(
        "Value match={} found={:#?} expected={:#?}",
        values_match, value, query_value
    );
    println!("Took {:?}", duration);

    Ok(values_match)
}
