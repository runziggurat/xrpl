//! Utilities for node configuration.

use std::{
    ffi::OsString,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde::Deserialize;

use crate::setup::{
    constants::{
        JSON_RPC_PORT, RIPPLED_DIR, RIPPLED_NODE_SEED, SYNTHETIC_NODE_PUBLIC_KEY,
        VALIDATORS_FILE_NAME, ZIGGURAT_CONFIG,
    },
    node::NodeConfig,
};

/// Convenience struct for reading Ziggurat's configuration file.
#[derive(Deserialize)]
struct ConfigFile {
    /// The absolute path of where to run the start command.
    path: PathBuf,
    /// The command to start the node.
    start_command: String,
}

/// The node metadata read from Ziggurat's configuration file.
#[derive(Debug, Clone)]
pub struct NodeMetaData {
    /// The absolute path of where to run the start command.
    pub path: PathBuf,
    /// The command to start the node.
    pub start_command: OsString,
    /// The arguments to the start command of the node.
    pub start_args: Vec<OsString>,
}

impl NodeMetaData {
    pub fn new(setup_path: PathBuf) -> Result<NodeMetaData> {
        // Read Ziggurat's configuration file.
        let path = setup_path.join(ZIGGURAT_CONFIG);
        let config_string = fs::read_to_string(path)?;
        let config_file: ConfigFile = toml::from_str(&config_string)?;

        // Read the args (which includes the start command at index 0).
        let args_from = |command: &str| -> Vec<OsString> {
            command.split_whitespace().map(OsString::from).collect()
        };

        // Separate the start command from the args list.
        let mut start_args = args_from(&config_file.start_command);
        let start_command = start_args.remove(0);

        Ok(Self {
            path: config_file.path,
            start_command,
            start_args,
        })
    }
}

pub struct RippledConfigFile;

impl RippledConfigFile {
    pub fn generate(config: &NodeConfig, path: &Path) -> Result<String> {
        let mut config_str = String::new();

        // 1. Server

        writeln!(&mut config_str, "[server]")?;
        writeln!(&mut config_str, "port_rpc_admin_local")?;
        writeln!(&mut config_str, "port_peer")?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[port_rpc_admin_local]")?;
        writeln!(&mut config_str, "port = {JSON_RPC_PORT}")?;
        writeln!(&mut config_str, "ip = {}", config.local_addr.ip())?;
        writeln!(&mut config_str, "admin = {}", config.local_addr.ip())?;
        writeln!(&mut config_str, "protocol = http")?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[port_peer]")?;
        writeln!(&mut config_str, "port = {}", config.local_addr.port())?;
        writeln!(&mut config_str, "ip = {}", config.local_addr.ip())?;
        writeln!(&mut config_str, "protocol = peer")?;
        writeln!(&mut config_str)?;

        if let Some(token) = &config.validator_token {
            writeln!(&mut config_str, "[validator_token]")?;
            writeln!(&mut config_str, "{token}")?;
            writeln!(&mut config_str)?;
        }

        if let Some(network_id) = &config.network_id {
            writeln!(&mut config_str, "[network_id]")?;
            writeln!(&mut config_str, "{network_id}")?;
            writeln!(&mut config_str)?;
        }

        // 2. Peer protocol
        writeln!(&mut config_str, "[reduce_relay]")?;
        writeln!(&mut config_str, "tx_enable = 1")?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[ledger_replay]")?;
        writeln!(&mut config_str, "1")?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[ips_fixed]")?;
        for addr in &config.initial_peers {
            writeln!(&mut config_str, "{} {}", addr.ip(), addr.port())?;
        }
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[peers_max]")?;
        writeln!(&mut config_str, "{}", config.max_peers)?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[sntp_servers]")?;
        writeln!(&mut config_str, "time.windows.com")?;
        writeln!(&mut config_str, "time.apple.com")?;
        writeln!(&mut config_str, "time.nist.gov")?;
        writeln!(&mut config_str, "pool.ntp.org")?;
        writeln!(&mut config_str)?;

        // 3. Ripple protocol
        if config.enable_cluster {
            writeln!(&mut config_str, "[node_seed]")?;
            writeln!(&mut config_str, "{RIPPLED_NODE_SEED}")?;
            writeln!(&mut config_str)?;

            writeln!(&mut config_str, "[cluster_nodes]")?;
            writeln!(&mut config_str, "{SYNTHETIC_NODE_PUBLIC_KEY}")?;
            writeln!(&mut config_str)?;
        }

        writeln!(&mut config_str, "[validators_file]")?;
        writeln!(&mut config_str, "{VALIDATORS_FILE_NAME}")?;
        writeln!(&mut config_str)?;

        if config.enable_sharding {
            writeln!(&mut config_str, "[shard_db]")?;
            writeln!(
                &mut config_str,
                "path={}",
                path.join(RIPPLED_DIR)
                    .join("db/shard/nudb")
                    .to_str()
                    .unwrap()
            )?;
            // For our test it's sufficient to hold the smallest possible number of shards.
            // More information about sharding config: https://xrpl.org/configure-history-sharding.html
            writeln!(&mut config_str, "max_historical_shards=1")?;
            writeln!(&mut config_str)?;
        }

        // 4. HTTPS client

        writeln!(&mut config_str, "[ssl_verify]")?;
        writeln!(&mut config_str, "0")?;
        writeln!(&mut config_str)?;

        // 5. Reporting mode

        // 6. Database

        writeln!(&mut config_str, "[node_db]")?;
        writeln!(&mut config_str, "type=NuDB")?;
        writeln!(
            &mut config_str,
            "path={}",
            path.join(RIPPLED_DIR).join("db/nudb").to_str().unwrap()
        )?;
        writeln!(&mut config_str, "online_delete=512")?;
        writeln!(&mut config_str, "advisory_delete=0")?;
        writeln!(&mut config_str)?;

        writeln!(&mut config_str, "[database_path]")?;
        writeln!(
            &mut config_str,
            "{}",
            path.join(RIPPLED_DIR).join("db").to_str().unwrap()
        )?;
        writeln!(&mut config_str)?;

        // 7. Diagnostics

        writeln!(&mut config_str, "[debug_logfile]")?;
        writeln!(
            &mut config_str,
            "{}",
            path.join(RIPPLED_DIR).join("debug.log").to_str().unwrap()
        )?;
        writeln!(&mut config_str)?;

        // 8. Voting

        // 9. Misc settings

        // 10. Example settings

        Ok(config_str)
    }
}
