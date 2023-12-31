#[cfg(feature = "impl_node")]
use std::borrow::Cow;
#[cfg(feature = "impl_rs")]
use std::net::ToSocketAddrs;
#[cfg(feature = "impl_node")]
use std::path::PathBuf;
#[cfg(feature = "impl_node")]
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::error::{Error, GenericError};
use crate::value::CommonValue;
use crate::QueryOptions;

pub trait QueryImplementation {
    fn query_server(&self, options: &QueryOptions) -> Result<CommonValue, GenericError>;
}

#[cfg(feature = "impl_rs")]
#[derive(Debug)]
pub struct RustImpl(gamedig::protocols::ExtraRequestSettings);
#[cfg(feature = "impl_rs")]
impl Default for RustImpl {
    fn default() -> Self {
        Self(
            gamedig::protocols::ExtraRequestSettings::default()
                .set_check_app_id(false)
                .set_gather_players(true)
                .set_gather_rules(false),
        )
    }
}
#[cfg(feature = "impl_rs")]
impl QueryImplementation for RustImpl {
    fn query_server(&self, options: &QueryOptions) -> Result<CommonValue, GenericError> {
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

        let timeout_settings = gamedig::protocols::types::TimeoutSettings::new(
            Some(Duration::from_secs(5)),
            Some(Duration::from_secs(5)),
            1,
        )?;

        let output = gamedig::query_with_timeout_and_extra_settings(
            game,
            &ip,
            options.port,
            Some(timeout_settings),
            Some(self.0.clone().set_hostname(options.address.clone())),
        )?;

        #[cfg(feature = "print_raw")]
        println!("{:#?}", output.as_json());

        Ok(output.as_json().into())
    }
}

#[cfg(feature = "impl_node")]
#[derive(Debug)]
pub struct NodeImpl {
    pub node_path: PathBuf,
    pub gamedig_path: PathBuf,
    pub node_args: Option<Vec<String>>,
}
#[cfg(feature = "impl_node")]
impl Default for NodeImpl {
    fn default() -> Self {
        Self {
            node_path: "node".into(),
            gamedig_path: "./node-gamedig/bin/gamedig.js".into(),
            node_args: None,
        }
    }
}
#[cfg(feature = "impl_node")]
impl QueryImplementation for NodeImpl {
    fn query_server(&self, options: &QueryOptions) -> Result<CommonValue, GenericError> {
        let mut host_str = Cow::from(&options.address);
        if let Some(port) = options.port {
            host_str.to_mut().push_str(&format!(":{}", port));
        }

        let mut command = Command::new(&self.node_path);
        command.stderr(Stdio::inherit());

        if let Some(node_args) = &self.node_args {
            command.args(node_args);
        }

        command
            .arg(&self.gamedig_path)
            .arg("--type")
            .arg(&options.game)
            .arg(host_str.as_ref());

        println!("Running {:?}", command);

        let output = command.output()?;

        if !output.status.success() {
            return Err(Error::String(
                String::from_utf8_lossy(&output.stdout).to_string(),
            ))?;
        }

        let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;

        #[cfg(feature = "print_raw")]
        println!("{:#?}", value);

        Ok(value.try_into()?)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Implementations {
    #[cfg(feature = "impl_node")]
    Node,
    #[cfg(feature = "impl_rs")]
    Rust,
}
