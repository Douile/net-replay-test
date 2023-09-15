use clap::{arg, value_parser, Command};

use net_replay_test::{capture, replay, QueryOptions};
use net_replay_test::{implementations::*, QueryReplay};

enum Mode {
    Capture,
    Replay,
}

impl TryFrom<String> for Mode {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "capture" => Ok(Mode::Capture),
            "replay" => Ok(Mode::Replay),
            _ => Err("Mode must be \"capture\" or \"replay\""),
        }
    }
}

fn main() {
    let mut command = clap::command!()
        .arg(arg!(-i --implementation <IMPL> "Optional implementation to use"))
        .subcommand(
            Command::new("capture")
                .about("Capture a new test (requires cap_net_raw,cap_net_admin=eip)")
                .arg(arg!(<game> "Name of game (to query)"))
                .arg(arg!(<address> "Hostname of server (to query)"))
                .arg(arg!([port] "Optional port (to query)").value_parser(value_parser!(u16)))
                .arg(arg!(-d --device <device> "Device to capture on")),
        )
        .subcommand(
            Command::new("replay")
                .about("Replay a captured test")
                .arg(arg!(<file> "Capture file")),
        );

    let matches = command.clone().get_matches();

    let implementation: Box<dyn QueryImplementation> =
        if let Some(impl_name) = matches.get_one::<String>("implementation") {
            match impl_name.as_str() {
                "node" => Box::new(NodeImpl::default()),
                "rust" => Box::new(RustImpl::default()),
                _ => panic!("No such impl {:?}", impl_name),
            }
        } else {
            Box::new(NodeImpl::default())
        };

    if let Some(matches) = matches.subcommand_matches("capture") {
        do_capture(implementation, matches);
    } else if let Some(matches) = matches.subcommand_matches("replay") {
        do_replay(implementation, matches)
    } else {
        let _ = command.print_help().unwrap();
    }
}

fn do_capture(i: Box<dyn QueryImplementation>, matches: &clap::ArgMatches) {
    let game = matches.get_one::<String>("game").unwrap();
    let address = matches.get_one::<String>("address").unwrap();
    let port = matches.get_one::<u16>("port");
    let device = matches.get_one::<String>("device");

    let opts = QueryOptions {
        game: game.to_string(),
        address: address.to_string(),
        port: port.copied(),
    };

    let replay_name = opts.as_file_name();

    let r = capture(i, opts, device.map(|x| x.as_str()));
    println!("{:#?}", r);

    let r = r.unwrap();

    let file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(replay_name)
        .unwrap();

    serde_json::to_writer(file, &r).unwrap();
}

fn do_replay(i: Box<dyn QueryImplementation>, matches: &clap::ArgMatches) {
    let file = matches.get_one::<String>("file").expect("Need file");

    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(file)
        .expect("File should exist");

    let query_replay: QueryReplay = serde_json::from_reader(file).expect("Invalid replay");

    replay(i, query_replay).unwrap();
}