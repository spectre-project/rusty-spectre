use crate::{block::Block, header::Header, subnets::SUBNETWORK_ID_COINBASE, tx::Transaction};
use spectre_hashes::{Hash, ZERO_HASH};
use spectre_muhash::EMPTY_MUHASH;

/// The constants uniquely representing the genesis block
#[derive(Clone, Debug)]
pub struct GenesisBlock {
    pub hash: Hash,
    pub version: u16,
    pub hash_merkle_root: Hash,
    pub utxo_commitment: Hash,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub coinbase_payload: &'static [u8],
}

impl GenesisBlock {
    pub fn build_genesis_transactions(&self) -> Vec<Transaction> {
        vec![Transaction::new(0, Vec::new(), Vec::new(), 0, SUBNETWORK_ID_COINBASE, 0, self.coinbase_payload.to_vec())]
    }
}

impl From<&GenesisBlock> for Header {
    fn from(genesis: &GenesisBlock) -> Self {
        Header::new_finalized(
            genesis.version,
            Vec::new(),
            genesis.hash_merkle_root,
            ZERO_HASH,
            genesis.utxo_commitment,
            genesis.timestamp,
            genesis.bits,
            genesis.nonce,
            genesis.daa_score,
            0.into(),
            0,
            ZERO_HASH,
        )
    }
}

impl From<&GenesisBlock> for Block {
    fn from(genesis: &GenesisBlock) -> Self {
        Block::new(genesis.into(), genesis.build_genesis_transactions())
    }
}

impl From<(&Header, &'static [u8])> for GenesisBlock {
    fn from((header, payload): (&Header, &'static [u8])) -> Self {
        Self {
            hash: header.hash,
            version: header.version,
            hash_merkle_root: header.hash_merkle_root,
            utxo_commitment: header.utxo_commitment,
            timestamp: header.timestamp,
            bits: header.bits,
            nonce: header.nonce,
            daa_score: header.daa_score,
            coinbase_payload: payload,
        }
    }
}

/// The genesis block of the block-DAG which serves as the public transaction ledger for the main network.
pub const GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x2A, 0xFA, 0x63, 0xE3, 0xAC, 0x16, 0x65, 0x62, 0x97, 0xBE, 0xF7, 0x75, 0x23, 0x79, 0x91, 0xC3, 0x9E, 0xED, 0x10, 0xF5, 0x23,
        0x84, 0xEE, 0x9D, 0x94, 0x20, 0x2C, 0x80, 0x1C, 0x76, 0xF5, 0x5D,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x55, 0x29, 0xA6, 0xB3, 0xB8, 0x7F, 0xC2, 0x09, 0x12, 0xA6, 0xE6, 0xD7, 0x9E, 0xFF, 0x9B, 0x92, 0x49, 0xF2, 0x4F, 0xF9, 0xED,
        0xDA, 0x4D, 0xEC, 0x40, 0x59, 0xEF, 0x9E, 0xD7, 0xC5, 0xBD, 0xCB,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1714369615432,
    bits: 536999497, // Prime number
    nonce: 271828,   // Euler's number
    daa_score: 0,    // Checkpoint DAA score
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00,                                     // Script version
        0x01,                                           // Varint
        0x00,                                           // OP-FALSE
        0x27, 0x18, 0x28, 0x18, 0x28, 0x45, 0x90, 0x45, // Euler's number = 2.718281828459045
    ],
};

pub const TESTNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x48, 0x44, 0xDF, 0x54, 0x95, 0x72, 0x66, 0x0E, 0xAF, 0xDC, 0x9A, 0xA0, 0xBC, 0x1D, 0x2B, 0xEE, 0xB8, 0xCA, 0x14, 0x0A, 0x5B,
        0x5D, 0x63, 0x15, 0xDC, 0x41, 0xBA, 0x42, 0x9B, 0xD2, 0x44, 0x00,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0xC5, 0xAE, 0xEF, 0x98, 0xF3, 0xE4, 0xF2, 0xBA, 0x2C, 0xB4, 0xAF, 0x00, 0xC1, 0x6F, 0xEC, 0x3D, 0x59, 0x9A, 0xF8, 0x03, 0x4E,
        0xE1, 0xE0, 0x15, 0xBC, 0x20, 0xCA, 0x60, 0xC9, 0x3E, 0x99, 0x1C,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1713884672545,
    bits: 511699987, // Prime number
    nonce: 314159,   // Pi number
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00,                                     // Script version
        0x01,                                           // Varint
        0x00,                                           // OP-FALSE
        0x31, 0x41, 0x59, 0x26, 0x53, 0x58, 0x97, 0x93, // Pi = 3.141592653589793
    ],
};

pub const TESTNET11_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0xAD, 0x64, 0xC6, 0x3F, 0x4B, 0xD6, 0xA9, 0x36, 0xA5, 0x2D, 0xE3, 0xFD, 0x26, 0x94, 0x74, 0x9D, 0x77, 0xFE, 0x7B, 0xD5, 0x96,
        0xE8, 0x46, 0xD8, 0x26, 0x90, 0xB7, 0xB4, 0xD7, 0xF5, 0x3C, 0x8F,
    ]),
    hash_merkle_root: Hash::from_bytes([
        0xD4, 0x08, 0xA5, 0xD2, 0xF6, 0x40, 0xC2, 0x75, 0x7D, 0x69, 0x84, 0x22, 0xF5, 0xEF, 0xFB, 0xD5, 0xF3, 0x9B, 0xA8, 0x79, 0x9D,
        0x2C, 0x1C, 0x8E, 0x74, 0xAA, 0x2B, 0x4D, 0xA4, 0x2E, 0xE0, 0x77,
    ]),
    bits: 504154830, // see `gen_testnet11_genesis`
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00,                                     // Script version
        0x01,                                           // Varint
        0x00,                                           // OP-FALSE
        0x6B, 0x61, 0x73, 0x70, 0x61, 0x2D, 0x74, 0x65,
    ],
    ..TESTNET_GENESIS
};

pub const SIMNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x56, 0xBB, 0x87, 0xCF, 0x18, 0x77, 0x7B, 0x76, 0x35, 0x8E, 0xEE, 0xF0, 0x20, 0xA9, 0x01, 0xCD, 0xDD, 0xDC, 0x0B, 0xA4, 0x46,
        0xC0, 0x99, 0x2D, 0xE2, 0x7C, 0xC2, 0xA8, 0x9E, 0xC7, 0xA1, 0x30,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x85, 0x81, 0x84, 0xD0, 0x98, 0x16, 0x40, 0x4F, 0xD7, 0xD7, 0x96, 0xFB, 0xDE, 0x60, 0xAC, 0x4B, 0x99, 0x29, 0xB9, 0x18, 0x63,
        0x39, 0xDA, 0x23, 0x08, 0x3C, 0xDF, 0xC3, 0x5F, 0x13, 0x8F, 0xC6,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1713885012324,
    bits: 543656363, // Prime number
    nonce: 2,        // Two
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00,                                     // Script version
        0x01,                                           // Varint
        0x00,                                           // OP-FALSE
        0x54, 0x36, 0x56, 0x36, 0x56, 0x91, 0x80, 0x90, // Euler's number * 2 = 5.436563656918090
    ],
};

pub const DEVNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x6C, 0x34, 0x89, 0xBF, 0xB5, 0x92, 0xCA, 0x0A, 0x0C, 0x12, 0xED, 0xB7, 0xAD, 0x86, 0x2D, 0x62, 0x27, 0x92, 0x3E, 0xC2, 0xD2,
        0x77, 0x7E, 0x0D, 0xFD, 0x93, 0xF3, 0xC5, 0xB8, 0xA5, 0x5C, 0x35,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x45, 0x7F, 0x6D, 0xF5, 0x76, 0x25, 0xCF, 0xC9, 0x4A, 0x63, 0x16, 0x9E, 0xBA, 0xC8, 0xE1, 0x86, 0xCF, 0x1B, 0x5F, 0x1E, 0xF6,
        0x8D, 0x1A, 0xEF, 0x3B, 0x8D, 0x3F, 0xFC, 0xC2, 0x6C, 0x01, 0xE4,
    ]),
    utxo_commitment: EMPTY_MUHASH,
    timestamp: 1713884849877,
    bits: 541034453, // Prime number
    nonce: 241421,   // Silver ratio
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Blue score
        0x00, 0xE1, 0xF5, 0x05, 0x00, 0x00, 0x00, 0x00, // Subsidy
        0x00, 0x00,                                     // Script version
        0x01,                                           // Varint
        0x00,                                           // OP-FALSE
        0x24, 0x14, 0x21, 0x35, 0x62, 0x37, 0x30, 0x95, // Silver ratio
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::bps::TenBps, merkle::calc_hash_merkle_root};

    #[test]
    fn test_genesis_hashes() {
        [GENESIS, TESTNET_GENESIS, TESTNET11_GENESIS, SIMNET_GENESIS, DEVNET_GENESIS].into_iter().for_each(|genesis| {
            let block: Block = (&genesis).into();
            assert_hashes_eq(calc_hash_merkle_root(block.transactions.iter(), false), block.header.hash_merkle_root);
            assert_hashes_eq(block.hash(), genesis.hash);
        });
    }

    #[test]
    fn gen_testnet11_genesis() {
        let bps = TenBps::bps();
        let mut genesis = TESTNET_GENESIS;
        let target = spectre_math::Uint256::from_compact_target_bits(genesis.bits);
        let scaled_target = target * bps / 100;
        let scaled_bits = scaled_target.compact_target_bits();
        genesis.bits = scaled_bits;
        if genesis.bits != TESTNET11_GENESIS.bits {
            panic!("Testnet 11: new bits: {}\nnew hash: {:#04x?}", scaled_bits, Block::from(&genesis).hash().as_bytes());
        }
    }

    fn assert_hashes_eq(got: Hash, expected: Hash) {
        if got != expected {
            // Special hex print to ease changing the genesis hash according to the print if needed
            panic!("Got hash {:#04x?} while expecting {:#04x?}", got.as_bytes(), expected.as_bytes());
        }
    }
}
