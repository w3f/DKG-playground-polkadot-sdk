pub mod notification;
pub(crate) mod gossip;
pub(crate) mod peers;


pub(crate) mod dkg_protocol_name {
	use array_bytes::bytes2hex;
	use sc_network::ProtocolName;

	/// DKG shares gossip protocol name.
	const GOSSIP_NAME: &str = "/dkg/2";

	/// Name of the shares gossip protocol used by DKG.
	///
	/// Must be registered towards the networking in order for DKG dealer/aggregator to properly function.
	pub fn gossip_protocol_name<Hash: AsRef<[u8]>>(
		genesis_hash: Hash,
		fork_id: Option<&str>,
	) -> ProtocolName {
		let genesis_hash = genesis_hash.as_ref();
		if let Some(fork_id) = fork_id {
			format!("/{}/{}{}", bytes2hex("", genesis_hash), fork_id, GOSSIP_NAME).into()
		} else {
			format!("/{}{}", bytes2hex("", genesis_hash), GOSSIP_NAME).into()
		}
	}
}

/// Returns the configuration value to put in
/// [`sc_network::config::FullNetworkConfiguration`].
/// TODO: These peers need to be set to be the collators for the specific DKG parachain INVESTIGATE how
pub fn dkg_peers_set_config<
    B: sp_runtime::traits::Block,
    N: sc_network::NetworkBackend<B, <B as sp_runtime::traits::Block>::Hash>
>(
    gossip_protocol_name: sc_network::ProtocolName,
	metrics: sc_network::service::NotificationMetrics,
	peer_store_handle: std::sync::Arc<dyn sc_network::peer_store::PeerStoreProvider>,
) -> (N::NotificationProtocolConfig, Box<dyn sc_network::NotificationService>) {
    N::notification_config(
        gossip_protocol_name,
        Vec::new(),
        1024 * 1024,
        None,
        sc_network::config::SetConfig {
            in_peers: 0,
            out_peers: 3,
            reserved_nodes: Vec::new(),
            non_reserved_mode: sc_network::config::NonReservedPeerMode::Accept
        },
        metrics,
        peer_store_handle
    )
}