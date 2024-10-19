use crate::protowire::{spectred_request, SpectredRequest, SpectredResponse};

impl From<spectred_request::Payload> for SpectredRequest {
    fn from(item: spectred_request::Payload) -> Self {
        SpectredRequest { id: 0, payload: Some(item) }
    }
}

impl AsRef<SpectredRequest> for SpectredRequest {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<SpectredResponse> for SpectredResponse {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub mod spectred_request_convert {
    use crate::protowire::*;
    use spectre_rpc_core::{RpcError, RpcResult};

    impl_into_spectred_request!(Shutdown);
    impl_into_spectred_request!(SubmitBlock);
    impl_into_spectred_request!(GetBlockTemplate);
    impl_into_spectred_request!(GetBlock);
    impl_into_spectred_request!(GetInfo);

    impl_into_spectred_request!(GetCurrentNetwork);
    impl_into_spectred_request!(GetPeerAddresses);
    impl_into_spectred_request!(GetSink);
    impl_into_spectred_request!(GetMempoolEntry);
    impl_into_spectred_request!(GetMempoolEntries);
    impl_into_spectred_request!(GetConnectedPeerInfo);
    impl_into_spectred_request!(AddPeer);
    impl_into_spectred_request!(SubmitTransaction);
    impl_into_spectred_request!(SubmitTransactionReplacement);
    impl_into_spectred_request!(GetSubnetwork);
    impl_into_spectred_request!(GetVirtualChainFromBlock);
    impl_into_spectred_request!(GetBlocks);
    impl_into_spectred_request!(GetBlockCount);
    impl_into_spectred_request!(GetBlockDagInfo);
    impl_into_spectred_request!(ResolveFinalityConflict);
    impl_into_spectred_request!(GetHeaders);
    impl_into_spectred_request!(GetUtxosByAddresses);
    impl_into_spectred_request!(GetBalanceByAddress);
    impl_into_spectred_request!(GetBalancesByAddresses);
    impl_into_spectred_request!(GetSinkBlueScore);
    impl_into_spectred_request!(Ban);
    impl_into_spectred_request!(Unban);
    impl_into_spectred_request!(EstimateNetworkHashesPerSecond);
    impl_into_spectred_request!(GetMempoolEntriesByAddresses);
    impl_into_spectred_request!(GetCoinSupply);
    impl_into_spectred_request!(Ping);
    impl_into_spectred_request!(GetMetrics);
    impl_into_spectred_request!(GetConnections);
    impl_into_spectred_request!(GetSystemInfo);
    impl_into_spectred_request!(GetServerInfo);
    impl_into_spectred_request!(GetSyncStatus);
    impl_into_spectred_request!(GetDaaScoreTimestampEstimate);
    impl_into_spectred_request!(GetFeeEstimate);
    impl_into_spectred_request!(GetFeeEstimateExperimental);

    impl_into_spectred_request!(NotifyBlockAdded);
    impl_into_spectred_request!(NotifyNewBlockTemplate);
    impl_into_spectred_request!(NotifyUtxosChanged);
    impl_into_spectred_request!(NotifyPruningPointUtxoSetOverride);
    impl_into_spectred_request!(NotifyFinalityConflict);
    impl_into_spectred_request!(NotifyVirtualDaaScoreChanged);
    impl_into_spectred_request!(NotifyVirtualChainChanged);
    impl_into_spectred_request!(NotifySinkBlueScoreChanged);

    macro_rules! impl_into_spectred_request {
        ($name:tt) => {
            paste::paste! {
                impl_into_spectred_request_ex!(spectre_rpc_core::[<$name Request>],[<$name RequestMessage>],[<$name Request>]);
            }
        };
    }

    use impl_into_spectred_request;

    macro_rules! impl_into_spectred_request_ex {
        // ($($core_struct:ident)::+, $($protowire_struct:ident)::+, $($variant:ident)::+) => {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<&$core_struct> for spectred_request::Payload {
                fn from(item: &$core_struct) -> Self {
                    Self::$variant(item.into())
                }
            }

            impl From<&$core_struct> for SpectredRequest {
                fn from(item: &$core_struct) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<$core_struct> for spectred_request::Payload {
                fn from(item: $core_struct) -> Self {
                    Self::$variant((&item).into())
                }
            }

            impl From<$core_struct> for SpectredRequest {
                fn from(item: $core_struct) -> Self {
                    Self { id: 0, payload: Some((&item).into()) }
                }
            }

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&spectred_request::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &spectred_request::Payload) -> RpcResult<Self> {
                    if let spectred_request::Payload::$variant(request) = item {
                        request.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError("Payload".to_string(), stringify!($variant).to_string()))
                    }
                }
            }

            impl TryFrom<&SpectredRequest> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &SpectredRequest) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError("SpectreRequest".to_string(), "Payload".to_string()))?
                        .try_into()
                }
            }

            impl From<$protowire_struct> for SpectredRequest {
                fn from(item: $protowire_struct) -> Self {
                    Self { id: 0, payload: Some(spectred_request::Payload::$variant(item)) }
                }
            }

            impl From<$protowire_struct> for spectred_request::Payload {
                fn from(item: $protowire_struct) -> Self {
                    spectred_request::Payload::$variant(item)
                }
            }
        };
    }
    use impl_into_spectred_request_ex;
}

pub mod spectred_response_convert {
    use crate::protowire::*;
    use spectre_rpc_core::{RpcError, RpcResult};

    impl_into_spectred_response!(Shutdown);
    impl_into_spectred_response!(SubmitBlock);
    impl_into_spectred_response!(GetBlockTemplate);
    impl_into_spectred_response!(GetBlock);
    impl_into_spectred_response!(GetInfo);
    impl_into_spectred_response!(GetCurrentNetwork);

    impl_into_spectred_response!(GetPeerAddresses);
    impl_into_spectred_response!(GetSink);
    impl_into_spectred_response!(GetMempoolEntry);
    impl_into_spectred_response!(GetMempoolEntries);
    impl_into_spectred_response!(GetConnectedPeerInfo);
    impl_into_spectred_response!(AddPeer);
    impl_into_spectred_response!(SubmitTransaction);
    impl_into_spectred_response!(SubmitTransactionReplacement);
    impl_into_spectred_response!(GetSubnetwork);
    impl_into_spectred_response!(GetVirtualChainFromBlock);
    impl_into_spectred_response!(GetBlocks);
    impl_into_spectred_response!(GetBlockCount);
    impl_into_spectred_response!(GetBlockDagInfo);
    impl_into_spectred_response!(ResolveFinalityConflict);
    impl_into_spectred_response!(GetHeaders);
    impl_into_spectred_response!(GetUtxosByAddresses);
    impl_into_spectred_response!(GetBalanceByAddress);
    impl_into_spectred_response!(GetBalancesByAddresses);
    impl_into_spectred_response!(GetSinkBlueScore);
    impl_into_spectred_response!(Ban);
    impl_into_spectred_response!(Unban);
    impl_into_spectred_response!(EstimateNetworkHashesPerSecond);
    impl_into_spectred_response!(GetMempoolEntriesByAddresses);
    impl_into_spectred_response!(GetCoinSupply);
    impl_into_spectred_response!(Ping);
    impl_into_spectred_response!(GetMetrics);
    impl_into_spectred_response!(GetConnections);
    impl_into_spectred_response!(GetSystemInfo);
    impl_into_spectred_response!(GetServerInfo);
    impl_into_spectred_response!(GetSyncStatus);
    impl_into_spectred_response!(GetDaaScoreTimestampEstimate);
    impl_into_spectred_response!(GetFeeEstimate);
    impl_into_spectred_response!(GetFeeEstimateExperimental);

    impl_into_spectred_notify_response!(NotifyBlockAdded);
    impl_into_spectred_notify_response!(NotifyNewBlockTemplate);
    impl_into_spectred_notify_response!(NotifyUtxosChanged);
    impl_into_spectred_notify_response!(NotifyPruningPointUtxoSetOverride);
    impl_into_spectred_notify_response!(NotifyFinalityConflict);
    impl_into_spectred_notify_response!(NotifyVirtualDaaScoreChanged);
    impl_into_spectred_notify_response!(NotifyVirtualChainChanged);
    impl_into_spectred_notify_response!(NotifySinkBlueScoreChanged);

    impl_into_spectred_notify_response!(NotifyUtxosChanged, StopNotifyingUtxosChanged);
    impl_into_spectred_notify_response!(NotifyPruningPointUtxoSetOverride, StopNotifyingPruningPointUtxoSetOverride);

    macro_rules! impl_into_spectred_response {
        ($name:tt) => {
            paste::paste! {
                impl_into_spectred_response_ex!(spectre_rpc_core::[<$name Response>],[<$name ResponseMessage>],[<$name Response>]);
            }
        };
        ($core_name:tt, $protowire_name:tt) => {
            paste::paste! {
                impl_into_spectred_response_base!(spectre_rpc_core::[<$core_name Response>],[<$protowire_name ResponseMessage>],[<$protowire_name Response>]);
            }
        };
    }
    use impl_into_spectred_response;

    macro_rules! impl_into_spectred_response_base {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<$core_struct>> for $protowire_struct {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    item.as_ref().map_err(|x| (*x).clone()).into()
                }
            }

            impl From<RpcError> for $protowire_struct {
                fn from(item: RpcError) -> Self {
                    let x: RpcResult<&$core_struct> = Err(item);
                    x.into()
                }
            }

            impl From<$protowire_struct> for spectred_response::Payload {
                fn from(item: $protowire_struct) -> Self {
                    spectred_response::Payload::$variant(item)
                }
            }

            impl From<$protowire_struct> for SpectredResponse {
                fn from(item: $protowire_struct) -> Self {
                    Self { id: 0, payload: Some(spectred_response::Payload::$variant(item)) }
                }
            }
        };
    }
    use impl_into_spectred_response_base;

    macro_rules! impl_into_spectred_response_ex {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<&$core_struct>> for spectred_response::Payload {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    spectred_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<&$core_struct>> for SpectredResponse {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl From<RpcResult<$core_struct>> for spectred_response::Payload {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    spectred_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<$core_struct>> for SpectredResponse {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    Self { id: 0, payload: Some(item.into()) }
                }
            }

            impl_into_spectred_response_base!($core_struct, $protowire_struct, $variant);

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&spectred_response::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &spectred_response::Payload) -> RpcResult<Self> {
                    if let spectred_response::Payload::$variant(response) = item {
                        response.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError("Payload".to_string(), stringify!($variant).to_string()))
                    }
                }
            }

            impl TryFrom<&SpectredResponse> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &SpectredResponse) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError("SpectreResponse".to_string(), "Payload".to_string()))?
                        .try_into()
                }
            }
        };
    }
    use impl_into_spectred_response_ex;

    macro_rules! impl_into_spectred_notify_response {
        ($name:tt) => {
            impl_into_spectred_response!($name);

            paste::paste! {
                impl_into_spectred_notify_response_ex!(spectre_rpc_core::[<$name Response>],[<$name ResponseMessage>]);
            }
        };
        ($core_name:tt, $protowire_name:tt) => {
            impl_into_spectred_response!($core_name, $protowire_name);

            paste::paste! {
                impl_into_spectred_notify_response_ex!(spectre_rpc_core::[<$core_name Response>],[<$protowire_name ResponseMessage>]);
            }
        };
    }
    use impl_into_spectred_notify_response;

    macro_rules! impl_into_spectred_notify_response_ex {
        ($($core_struct:ident)::+, $protowire_struct:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl<T> From<Result<(), T>> for $protowire_struct
            where
                T: Into<RpcError>,
            {
                fn from(item: Result<(), T>) -> Self {
                    item
                        .map(|_| $($core_struct)::+{})
                        .map_err(|err| err.into()).into()
                }
            }

        };
    }
    use impl_into_spectred_notify_response_ex;
}
