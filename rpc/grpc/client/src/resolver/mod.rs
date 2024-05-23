use super::error::Result;
use core::fmt::Debug;
use spectre_grpc_core::{
    ops::SpectredPayloadOps,
    protowire::{SpectredRequest, SpectredResponse},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::oneshot;

pub(crate) mod id;
pub(crate) mod matcher;
pub(crate) mod queue;

pub(crate) trait Resolver: Send + Sync + Debug {
    fn register_request(&self, op: SpectredPayloadOps, request: &SpectredRequest) -> SpectredResponseReceiver;
    fn handle_response(&self, response: SpectredResponse);
    fn remove_expired_requests(&self, timeout: Duration);
}

pub(crate) type DynResolver = Arc<dyn Resolver>;

pub(crate) type SpectredResponseSender = oneshot::Sender<Result<SpectredResponse>>;
pub(crate) type SpectredResponseReceiver = oneshot::Receiver<Result<SpectredResponse>>;
