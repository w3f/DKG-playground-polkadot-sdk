// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use std::net::Incoming;

use futures::{future::Either, FutureExt, StreamExt, TryFutureExt};

use polkadot_node_network_protocol::{
	request_response::{v1, v2, IncomingRequest, IncomingRequestReceiver, ReqProtocolNames,},
	UnifiedReputationChange as Rep
};
use polkadot_node_subsystem::{
	jaeger, messages::DkgMessage, overseer, FromOrchestra, OverseerSignal,
	SpawnedSubsystem, SubsystemError, SubsystemSender,
};
use fatality::Nested;

const LOG_TARGET: &'static str = "validator::dkg-share";

#[allow(missing_docs)]
#[fatality::fatality(splitable)]
pub enum DkgError {
	#[fatal]
	#[error("Spawning subsystem Dkg Task failed: {0}")]
	SpawnDkgTask(#[source] SubsystemError)
}

const COST_INVALID_REQUEST: Rep = Rep::CostMajor("Beep Bop DKG");

async fn run_dkg_share_receiver<Sender>(
	mut sender: Sender,
	mut receiver: IncomingRequestReceiver<v2::DkgShareRequest>,
) where
	Sender: SubsystemSender<DkgMessage>
{
	loop {
		match receiver.recv(|| vec![COST_INVALID_REQUEST]).await.into_nested() {
			Ok(Ok(request)) =>  {
				// TODO: Actually modulate this as a function who correctly formulates the DkgShare response
				// Look through storage and do something basically
				let response = v2::DkgShareResponse::Share(vec![1]);
				if let Err(err) = request.send_response(response) {
					gum::warn!(
						target: LOG_TARGET,
						err= ?err,
						"Dkg Share Response failed"
					);
				}
			},
			Err(fatal) => {
				gum::debug!(
					target: LOG_TARGET,
					error = ?fatal,
					"Shutting down dkg share receiver."
				);
			},
			Ok(Err(jyfi)) => {
				gum::debug!(
					target: LOG_TARGET,
					error = ?jyfi,
					"Error decoding incoming DkgShare Request."
				);
			},
		}
	}

}

// TODO: Add struct for DkgSubsystem
pub struct DkgSubsystem {
	/// Receivers to receive messages from.
	recvs: IncomingRequestReceivers,
	/// Mapping of the req-response protocols to the full protocol names.
	req_protocol_names: ReqProtocolNames,
}

pub struct IncomingRequestReceivers {
	pub dkg_receiver: Option<IncomingRequestReceiver<v2::DkgShareRequest>>
}

// TODO: implement overseer proc macro for subsystem with a start function
#[overseer::subsystem(DkgProtocol, error=SubsystemError, prefix=self::overseer)]
impl<Context> DkgSubsystem {
	fn start(self, ctx: Context) -> SpawnedSubsystem {
		let future = self
			.run(ctx)
			.map_err(|e| SubsystemError::with_origin("dkg system", e))
			.boxed();
		SpawnedSubsystem { name: "dkg-subsystem", future }
	}
}

#[overseer::contextbounds(DkgProtocol, prefix = self::overseer)]
impl DkgSubsystem {
	pub fn new(recvs: IncomingRequestReceivers, req_protocol_names: ReqProtocolNames) -> Self {
		Self { recvs, req_protocol_names }
	}

	async fn run<Context>(self, mut ctx: Context) -> std::result::Result<(), DkgError> {
		let Self { recvs, req_protocol_names } = self;

		let IncomingRequestReceivers { dkg_receiver } = recvs;

		if let Some(recv) = dkg_receiver
		{
			// TODO:
			// Get sender from ctx
			let sender = ctx.sender().clone();
			ctx.spawn(
				"dkg share receiver",
				run_dkg_share_receiver(sender, recv).boxed(),
			)
			.map_err(FatalDkgError::SpawnDkgTask)?;
		}

		loop {
			// TODO:
			// get recv from ctx and action to do
			// Handle messages for sending from action
			// match on message and formulate the dkg request message
		}
	}
}
// Handle sending responses and receiving requests
