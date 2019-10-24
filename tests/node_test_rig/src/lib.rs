mod remote_node;

use beacon_node::{
    beacon_chain::{builder::BeaconChainStartMethod, BeaconChainTypes},
    Client, ClientConfig, ProductionBeaconNode, ProductionClient,
};
use environment::RuntimeContext;
use genesis::interop_genesis_state;
use tempdir::TempDir;
use types::{test_utils::generate_deterministic_keypairs, EthSpec};

pub use environment;
pub use remote_node::RemoteBeaconNode;

pub struct LocalBeaconNode<T> {
    pub client: T,
    pub datadir: TempDir,
}

impl<E: EthSpec> LocalBeaconNode<ProductionClient<E>> {
    pub fn production(context: RuntimeContext<E>) -> Self {
        let (client_config, datadir) = testing_client_config();
        let eth2_config = context.eth2_config().clone();

        let state = interop_genesis_state(
            &generate_deterministic_keypairs(8),
            0,
            &context.eth2_config().spec,
        )
        .expect("should build interop state");

        let client = ProductionBeaconNode::from_genesis(context, state, client_config, eth2_config)
            .expect("should build production client")
            .into_inner();

        LocalBeaconNode { client, datadir }
    }
}

impl<T: BeaconChainTypes> LocalBeaconNode<Client<T>> {
    pub fn remote_node(&self) -> Result<RemoteBeaconNode<T::EthSpec>, String> {
        Ok(RemoteBeaconNode::new(
            self.client
                .http_listen_addr()
                .ok_or_else(|| "A remote beacon node must have a http server".to_string())?,
        )?)
    }
}

fn testing_client_config() -> (ClientConfig, TempDir) {
    // Creates a temporary directory that will be deleted once this `TempDir` is dropped.
    let tempdir = TempDir::new("lighthouse_node_test_rig")
        .expect("should create temp directory for client datadir");

    let mut client_config = ClientConfig::default();

    client_config.data_dir = tempdir.path().into();

    client_config.beacon_chain_start_method = BeaconChainStartMethod::Generated {
        validator_count: 8,
        genesis_time: 13371377,
    };

    // Setting ports to `0` means that the OS will choose some available port.
    client_config.network.libp2p_port = 0;
    client_config.network.discovery_port = 0;
    client_config.rpc.port = 0;
    client_config.rest_api.port = 0;
    client_config.websocket_server.port = 0;

    (client_config, tempdir)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}