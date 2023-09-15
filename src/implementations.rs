#[cfg(feature = "impl_rs")]
use std::net::ToSocketAddrs;
#[cfg(feature = "impl_node")]
use std::path::PathBuf;
#[cfg(feature = "impl_node")]
use std::process::Command;
use std::process::Stdio;

use crate::error::Error;
use crate::QueryOptions;

pub trait QueryImplementation {
    fn query_server(&self, options: &QueryOptions) -> Result<serde_json::value::Value, Error>;
}

#[cfg(feature = "impl_rs")]
#[derive(Debug)]
pub struct RustImpl(gamedig::protocols::ExtraRequestSettings);
#[cfg(feature = "impl_rs")]
impl Default for RustImpl {
    fn default() -> Self {
        Self(gamedig::protocols::ExtraRequestSettings::default().set_check_app_id(false))
    }
}
#[cfg(feature = "impl_rs")]
impl QueryImplementation for RustImpl {
    fn query_server(&self, options: &QueryOptions) -> Result<serde_json::value::Value, Error> {
        let game = gamedig::GAMES
            .get(&options.game)
            .ok_or(Error::String("Unknown game".to_string()))?;

        let ip = format!("{}:0", options.address)
            .to_socket_addrs()?
            .next()
            .ok_or(Error::String(
                "Given hostname did not resolve to an IP".to_string(),
            ))?
            .ip();

        let output = gamedig::query_with_timeout_and_extra_settings(
            &game,
            &ip,
            options.port,
            None,
            Some(self.0.clone().set_hostname(options.address.clone())),
        )
        .unwrap();

        Ok(serde_json::json!(output.as_json()))
    }
}

#[cfg(feature = "impl_node")]
#[derive(Debug)]
pub struct NodeImpl {
    pub node_path: PathBuf,
    pub gamedig_path: PathBuf,
}
#[cfg(feature = "impl_node")]
impl Default for NodeImpl {
    fn default() -> Self {
        Self {
            node_path: "node".into(),
            gamedig_path: "./node-gamedig/bin/gamedig.js".into(),
        }
    }
}
#[cfg(feature = "impl_node")]
impl QueryImplementation for NodeImpl {
    fn query_server(&self, options: &QueryOptions) -> Result<serde_json::value::Value, Error> {
        let mut host_str: String = format!("{}", options.address);
        if let Some(port) = options.port {
            host_str.push_str(&format!(":{}", port));
        }

        println!("Running {:?} {:?}", self.node_path, self.gamedig_path);

        let output = Command::new(&self.node_path)
            .arg(&self.gamedig_path)
            .arg("--type")
            .arg(&options.game)
            .arg(host_str)
            .stderr(Stdio::inherit())
            .output()?;

        let value = serde_json::from_slice(&output.stdout)?;

        Ok(value)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Implementations {
    #[cfg(feature = "impl_node")]
    Node,
    #[cfg(feature = "impl_rs")]
    Rust,
}
