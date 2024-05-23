use super::method::{DropFn, Method, MethodTrait, RoutingPolicy};
use crate::{
    connection::Connection,
    connection_handler::ServerContext,
    error::{GrpcServerError, GrpcServerResult},
};
use spectre_grpc_core::{
    ops::SpectredPayloadOps,
    protowire::{SpectredRequest, SpectredResponse},
};
use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

pub type SpectredMethod = Method<ServerContext, Connection, SpectredRequest, SpectredResponse>;
pub type DynSpectredMethod = Arc<dyn MethodTrait<ServerContext, Connection, SpectredRequest, SpectredResponse>>;
pub type SpectredDropFn = DropFn<SpectredRequest, SpectredResponse>;
pub type SpectredRoutingPolicy = RoutingPolicy<SpectredRequest, SpectredResponse>;

/// An interface providing methods implementations and a fallback "not implemented" method
/// actually returning a message with a "not implemented" error.
///
/// The interface can provide a method clone for every [`SpectredPayloadOps`] variant for later
/// processing of related requests.
///
/// It is also possible to directly let the interface itself process a request by invoking
/// the `call()` method.
pub struct Interface {
    server_ctx: ServerContext,
    methods: HashMap<SpectredPayloadOps, DynSpectredMethod>,
    method_not_implemented: DynSpectredMethod,
}

impl Interface {
    pub fn new(server_ctx: ServerContext) -> Self {
        let method_not_implemented = Arc::new(Method::new(|_, _, spectred_request: SpectredRequest| {
            Box::pin(async move {
                match spectred_request.payload {
                    Some(ref request) => Ok(SpectredResponse {
                        id: spectred_request.id,
                        payload: Some(
                            SpectredPayloadOps::from(request).to_error_response(GrpcServerError::MethodNotImplemented.into()),
                        ),
                    }),
                    None => Err(GrpcServerError::InvalidRequestPayload),
                }
            })
        }));
        Self { server_ctx, methods: Default::default(), method_not_implemented }
    }

    pub fn method(&mut self, op: SpectredPayloadOps, method: SpectredMethod) {
        let method: DynSpectredMethod = Arc::new(method);
        if self.methods.insert(op, method).is_some() {
            panic!("RPC method {op:?} is declared multiple times")
        }
    }

    pub fn replace_method(&mut self, op: SpectredPayloadOps, method: SpectredMethod) {
        let method: DynSpectredMethod = Arc::new(method);
        let _ = self.methods.insert(op, method);
    }

    pub fn set_method_properties(
        &mut self,
        op: SpectredPayloadOps,
        tasks: usize,
        queue_size: usize,
        routing_policy: SpectredRoutingPolicy,
    ) {
        self.methods.entry(op).and_modify(|x| {
            let method: Method<ServerContext, Connection, SpectredRequest, SpectredResponse> =
                Method::with_properties(x.method_fn(), tasks, queue_size, routing_policy);
            let method: Arc<dyn MethodTrait<ServerContext, Connection, SpectredRequest, SpectredResponse>> = Arc::new(method);
            *x = method;
        });
    }

    pub async fn call(
        &self,
        op: &SpectredPayloadOps,
        connection: Connection,
        request: SpectredRequest,
    ) -> GrpcServerResult<SpectredResponse> {
        self.methods.get(op).unwrap_or(&self.method_not_implemented).call(self.server_ctx.clone(), connection, request).await
    }

    pub fn get_method(&self, op: &SpectredPayloadOps) -> DynSpectredMethod {
        self.methods.get(op).unwrap_or(&self.method_not_implemented).clone()
    }
}

impl Debug for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interface").finish()
    }
}
