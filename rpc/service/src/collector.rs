use crate::converter::{consensus::ConsensusConverter, index::IndexConverter};
use spectre_notify::collector::CollectorFrom;

pub(crate) type CollectorFromConsensus = CollectorFrom<ConsensusConverter>;

pub(crate) type CollectorFromIndex = CollectorFrom<IndexConverter>;
