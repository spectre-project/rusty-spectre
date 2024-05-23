use crate::pb::spectred_message::Payload as SpectredMessagePayload;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum SpectredMessagePayloadType {
    Addresses = 0,
    Block,
    Transaction,
    BlockLocator,
    RequestAddresses,
    RequestRelayBlocks,
    RequestTransactions,
    IbdBlock,
    InvRelayBlock,
    InvTransactions,
    Ping,
    Pong,
    Verack,
    Version,
    TransactionNotFound,
    Reject,
    PruningPointUtxoSetChunk,
    RequestIbdBlocks,
    UnexpectedPruningPoint,
    IbdBlockLocator,
    IbdBlockLocatorHighestHash,
    RequestNextPruningPointUtxoSetChunk,
    DonePruningPointUtxoSetChunks,
    IbdBlockLocatorHighestHashNotFound,
    BlockWithTrustedData,
    DoneBlocksWithTrustedData,
    RequestPruningPointAndItsAnticone,
    BlockHeaders,
    RequestNextHeaders,
    DoneHeaders,
    RequestPruningPointUtxoSet,
    RequestHeaders,
    RequestBlockLocator,
    PruningPoints,
    RequestPruningPointProof,
    PruningPointProof,
    Ready,
    BlockWithTrustedDataV4,
    TrustedData,
    RequestIbdChainBlockLocator,
    IbdChainBlockLocator,
    RequestAntipast,
    RequestNextPruningPointAndItsAnticoneBlocks,
}

impl From<&SpectredMessagePayload> for SpectredMessagePayloadType {
    fn from(payload: &SpectredMessagePayload) -> Self {
        match payload {
            SpectredMessagePayload::Addresses(_) => SpectredMessagePayloadType::Addresses,
            SpectredMessagePayload::Block(_) => SpectredMessagePayloadType::Block,
            SpectredMessagePayload::Transaction(_) => SpectredMessagePayloadType::Transaction,
            SpectredMessagePayload::BlockLocator(_) => SpectredMessagePayloadType::BlockLocator,
            SpectredMessagePayload::RequestAddresses(_) => SpectredMessagePayloadType::RequestAddresses,
            SpectredMessagePayload::RequestRelayBlocks(_) => SpectredMessagePayloadType::RequestRelayBlocks,
            SpectredMessagePayload::RequestTransactions(_) => SpectredMessagePayloadType::RequestTransactions,
            SpectredMessagePayload::IbdBlock(_) => SpectredMessagePayloadType::IbdBlock,
            SpectredMessagePayload::InvRelayBlock(_) => SpectredMessagePayloadType::InvRelayBlock,
            SpectredMessagePayload::InvTransactions(_) => SpectredMessagePayloadType::InvTransactions,
            SpectredMessagePayload::Ping(_) => SpectredMessagePayloadType::Ping,
            SpectredMessagePayload::Pong(_) => SpectredMessagePayloadType::Pong,
            SpectredMessagePayload::Verack(_) => SpectredMessagePayloadType::Verack,
            SpectredMessagePayload::Version(_) => SpectredMessagePayloadType::Version,
            SpectredMessagePayload::TransactionNotFound(_) => SpectredMessagePayloadType::TransactionNotFound,
            SpectredMessagePayload::Reject(_) => SpectredMessagePayloadType::Reject,
            SpectredMessagePayload::PruningPointUtxoSetChunk(_) => SpectredMessagePayloadType::PruningPointUtxoSetChunk,
            SpectredMessagePayload::RequestIbdBlocks(_) => SpectredMessagePayloadType::RequestIbdBlocks,
            SpectredMessagePayload::UnexpectedPruningPoint(_) => SpectredMessagePayloadType::UnexpectedPruningPoint,
            SpectredMessagePayload::IbdBlockLocator(_) => SpectredMessagePayloadType::IbdBlockLocator,
            SpectredMessagePayload::IbdBlockLocatorHighestHash(_) => SpectredMessagePayloadType::IbdBlockLocatorHighestHash,
            SpectredMessagePayload::RequestNextPruningPointUtxoSetChunk(_) => {
                SpectredMessagePayloadType::RequestNextPruningPointUtxoSetChunk
            }
            SpectredMessagePayload::DonePruningPointUtxoSetChunks(_) => SpectredMessagePayloadType::DonePruningPointUtxoSetChunks,
            SpectredMessagePayload::IbdBlockLocatorHighestHashNotFound(_) => {
                SpectredMessagePayloadType::IbdBlockLocatorHighestHashNotFound
            }
            SpectredMessagePayload::BlockWithTrustedData(_) => SpectredMessagePayloadType::BlockWithTrustedData,
            SpectredMessagePayload::DoneBlocksWithTrustedData(_) => SpectredMessagePayloadType::DoneBlocksWithTrustedData,
            SpectredMessagePayload::RequestPruningPointAndItsAnticone(_) => {
                SpectredMessagePayloadType::RequestPruningPointAndItsAnticone
            }
            SpectredMessagePayload::BlockHeaders(_) => SpectredMessagePayloadType::BlockHeaders,
            SpectredMessagePayload::RequestNextHeaders(_) => SpectredMessagePayloadType::RequestNextHeaders,
            SpectredMessagePayload::DoneHeaders(_) => SpectredMessagePayloadType::DoneHeaders,
            SpectredMessagePayload::RequestPruningPointUtxoSet(_) => SpectredMessagePayloadType::RequestPruningPointUtxoSet,
            SpectredMessagePayload::RequestHeaders(_) => SpectredMessagePayloadType::RequestHeaders,
            SpectredMessagePayload::RequestBlockLocator(_) => SpectredMessagePayloadType::RequestBlockLocator,
            SpectredMessagePayload::PruningPoints(_) => SpectredMessagePayloadType::PruningPoints,
            SpectredMessagePayload::RequestPruningPointProof(_) => SpectredMessagePayloadType::RequestPruningPointProof,
            SpectredMessagePayload::PruningPointProof(_) => SpectredMessagePayloadType::PruningPointProof,
            SpectredMessagePayload::Ready(_) => SpectredMessagePayloadType::Ready,
            SpectredMessagePayload::BlockWithTrustedDataV4(_) => SpectredMessagePayloadType::BlockWithTrustedDataV4,
            SpectredMessagePayload::TrustedData(_) => SpectredMessagePayloadType::TrustedData,
            SpectredMessagePayload::RequestIbdChainBlockLocator(_) => SpectredMessagePayloadType::RequestIbdChainBlockLocator,
            SpectredMessagePayload::IbdChainBlockLocator(_) => SpectredMessagePayloadType::IbdChainBlockLocator,
            SpectredMessagePayload::RequestAntipast(_) => SpectredMessagePayloadType::RequestAntipast,
            SpectredMessagePayload::RequestNextPruningPointAndItsAnticoneBlocks(_) => {
                SpectredMessagePayloadType::RequestNextPruningPointAndItsAnticoneBlocks
            }
        }
    }
}
