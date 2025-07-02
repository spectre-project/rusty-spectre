/// BLOCK_VERSION represents the current block version
pub const BLOCK_VERSION: u16 = BLOCK_VERSION_SPECTREXV1;

pub const BLOCK_VERSION_SPECTREXV1: u16 = 1;
pub const BLOCK_VERSION_SPECTREXV2: u16 = 2;

/// TX_VERSION is the current latest supported transaction version.
pub const TX_VERSION: u16 = 0;

pub const LOCK_TIME_THRESHOLD: u64 = 500_000_000_000;

/// MAX_SCRIPT_PUBLIC_KEY_VERSION is the current latest supported public key script version.
pub const MAX_SCRIPT_PUBLIC_KEY_VERSION: u16 = 0;

/// SompiPerSpectre is the number of sompi in one spectre (1 SPR).
pub const SOMPI_PER_SPECTRE: u64 = 100_000_000;

/// The parameter for scaling inverse SPR value to mass units (KIP-0009)
pub const STORAGE_MASS_PARAMETER: u64 = SOMPI_PER_SPECTRE * 10_000;

/// The parameter defining how much mass per byte to charge for when calculating
/// transient storage mass. Since normally the block mass limit is 500_000, this limits
/// block body byte size to 125_000 (KIP-0013). We use a factor of 3 instead of 4 to
/// allow larger blocks and accommodate more transactions per block at our reduced BPS.
/// worst_case_usage = ((pruning_depth + finality_depth) * block_mass_limit) / bytes_per_gb;
///                    ((585128 + 259200) * 166667) / 1000000000 = 140.72 GB
pub const TRANSIENT_BYTE_TO_MASS_FACTOR: u64 = 3; // 167KB Block Size

/// MaxSompi is the maximum transaction amount allowed in sompi.
pub const MAX_SOMPI: u64 = 1_161_000_000 * SOMPI_PER_SPECTRE;

// MAX_TX_IN_SEQUENCE_NUM is the maximum sequence number the sequence field
// of a transaction input can be.
pub const MAX_TX_IN_SEQUENCE_NUM: u64 = u64::MAX;

// SEQUENCE_LOCK_TIME_MASK is a mask that extracts the relative lock time
// when masked against the transaction input sequence number.
pub const SEQUENCE_LOCK_TIME_MASK: u64 = 0x00000000ffffffff;

// SEQUENCE_LOCK_TIME_DISABLED is a flag that if set on a transaction
// input's sequence number, the sequence number will not be interpreted
// as a relative lock time.
pub const SEQUENCE_LOCK_TIME_DISABLED: u64 = 1 << 63;

/// UNACCEPTED_DAA_SCORE is used to for UtxoEntries that were created by
/// transactions in the mempool, or otherwise not-yet-accepted transactions.
pub const UNACCEPTED_DAA_SCORE: u64 = u64::MAX;
