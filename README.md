# net-replay-test

A testing tool for capturing network traffic to a specific server during a
query, then replaying the packets and validating the output matches.

## Requirements
### libpcap (for capture)

See [rust-pcap](https://github.com/rust-pcap/pcap#installing-dependencies) for installation help.

### `cap_net_raw,cap_net_admin=eip` (for capture)

In order to capture packets the binary requires special permissions on linux, to set use:

```shell
$ sudo setcap cap_net_raw,cap_net_admin=eip ./net-replay-test
```

## Usage (CLI)
### Build

```shell
$ cargo build --bin net-replay-test --all-features --release
```

### Capturing

```shell
$ ./net-replay-test --implementation node capture --device eth0 csgo 127.0.0.1 27015
```

A new JSON file named with the date, game, and hostname will be created in the
current directory if the capture was successful.

### Replay

```shell
$ ./net-replay-test --implementation node replay ./replay-...json
```

## Usage (test lib)
TODO
