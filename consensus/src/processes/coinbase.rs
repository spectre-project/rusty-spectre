use spectre_consensus_core::{
    coinbase::*,
    errors::coinbase::{CoinbaseError, CoinbaseResult},
    subnets,
    tx::{ScriptPublicKey, ScriptVec, Transaction, TransactionOutput},
    BlockHashMap, BlockHashSet,
};
use std::convert::TryInto;

use crate::{constants, model::stores::ghostdag::GhostdagData};

const LENGTH_OF_BLUE_SCORE: usize = size_of::<u64>();
const LENGTH_OF_SUBSIDY: usize = size_of::<u64>();
const LENGTH_OF_SCRIPT_PUB_KEY_VERSION: usize = size_of::<u16>();
const LENGTH_OF_SCRIPT_PUB_KEY_LENGTH: usize = size_of::<u8>();

const MIN_PAYLOAD_LENGTH: usize =
    LENGTH_OF_BLUE_SCORE + LENGTH_OF_SUBSIDY + LENGTH_OF_SCRIPT_PUB_KEY_VERSION + LENGTH_OF_SCRIPT_PUB_KEY_LENGTH;

// We define a year as 365.25 days and a month as 365.25 / 12 = 30.4375
// SECONDS_PER_MONTH = 30.4375 * 24 * 60 * 60
const SECONDS_PER_MONTH: u64 = 2629800;

pub const SUBSIDY_BY_MONTH_TABLE_SIZE: usize = 727;
pub type SubsidyByMonthTable = [u64; SUBSIDY_BY_MONTH_TABLE_SIZE];

#[derive(Clone)]
pub struct CoinbaseManager {
    coinbase_payload_script_public_key_max_len: u8,
    max_coinbase_payload_len: usize,
    deflationary_phase_daa_score: u64,
    pre_deflationary_phase_base_subsidy: u64,
    target_time_per_block: u64,

    /// Precomputed number of blocks per month
    blocks_per_month: u64,

    /// Precomputed subsidy by month table
    subsidy_by_month_table: SubsidyByMonthTable,
}

/// Struct used to streamline payload parsing
struct PayloadParser<'a> {
    remaining: &'a [u8], // The unparsed remainder
}

impl<'a> PayloadParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { remaining: data }
    }

    /// Returns a slice with the first `n` bytes of `remaining`, while setting `remaining` to the remaining part
    fn take(&mut self, n: usize) -> &[u8] {
        let (segment, remaining) = self.remaining.split_at(n);
        self.remaining = remaining;
        segment
    }
}

impl CoinbaseManager {
    pub fn new(
        coinbase_payload_script_public_key_max_len: u8,
        max_coinbase_payload_len: usize,
        deflationary_phase_daa_score: u64,
        pre_deflationary_phase_base_subsidy: u64,
        target_time_per_block: u64,
    ) -> Self {
        assert!(1000 % target_time_per_block == 0);
        let bps = 1000 / target_time_per_block;
        let blocks_per_month = SECONDS_PER_MONTH * bps;

        // Precomputed subsidy by month table for the actual block per second rate
        // Here values are rounded up so that we keep the same number of rewarding months as in the original 1 BPS table.
        // In a 10 BPS network, the induced increase in total rewards is 51 SPR (see tests::calc_high_bps_total_rewards_delta())
        let subsidy_by_month_table: SubsidyByMonthTable = core::array::from_fn(|i| (SUBSIDY_BY_MONTH_TABLE[i] + bps - 1) / bps);
        Self {
            coinbase_payload_script_public_key_max_len,
            max_coinbase_payload_len,
            deflationary_phase_daa_score,
            pre_deflationary_phase_base_subsidy,
            target_time_per_block,
            blocks_per_month,
            subsidy_by_month_table,
        }
    }

    #[cfg(test)]
    #[inline]
    pub fn bps(&self) -> u64 {
        1000 / self.target_time_per_block
    }

    pub fn expected_coinbase_transaction<T: AsRef<[u8]>>(
        &self,
        daa_score: u64,
        miner_data: MinerData<T>,
        ghostdag_data: &GhostdagData,
        mergeset_rewards: &BlockHashMap<BlockRewardData>,
        mergeset_non_daa: &BlockHashSet,
    ) -> CoinbaseResult<CoinbaseTransactionTemplate> {
        let mut outputs = Vec::with_capacity(ghostdag_data.mergeset_blues.len() + 1); // + 1 for possible red reward

        // Add an output for each mergeset blue block (∩ DAA window), paying to the script reported by the block.
        // Note that combinatorically it is nearly impossible for a blue block to be non-DAA
        for blue in ghostdag_data.mergeset_blues.iter().filter(|h| !mergeset_non_daa.contains(h)) {
            let reward_data = mergeset_rewards.get(blue).unwrap();
            if reward_data.subsidy + reward_data.total_fees > 0 {
                outputs
                    .push(TransactionOutput::new(reward_data.subsidy + reward_data.total_fees, reward_data.script_public_key.clone()));
            }
        }

        // Collect all rewards from mergeset reds ∩ DAA window and create a
        // single output rewarding all to the current block (the "merging" block)
        let mut red_reward = 0u64;
        for red in ghostdag_data.mergeset_reds.iter().filter(|h| !mergeset_non_daa.contains(h)) {
            let reward_data = mergeset_rewards.get(red).unwrap();
            red_reward += reward_data.subsidy + reward_data.total_fees;
        }
        if red_reward > 0 {
            outputs.push(TransactionOutput::new(red_reward, miner_data.script_public_key.clone()));
        }

        // Build the current block's payload
        let subsidy = self.calc_block_subsidy(daa_score);
        let payload = self.serialize_coinbase_payload(&CoinbaseData { blue_score: ghostdag_data.blue_score, subsidy, miner_data })?;

        Ok(CoinbaseTransactionTemplate {
            tx: Transaction::new(constants::TX_VERSION, vec![], outputs, 0, subnets::SUBNETWORK_ID_COINBASE, 0, payload),
            has_red_reward: red_reward > 0,
        })
    }

    pub fn serialize_coinbase_payload<T: AsRef<[u8]>>(&self, data: &CoinbaseData<T>) -> CoinbaseResult<Vec<u8>> {
        let script_pub_key_len = data.miner_data.script_public_key.script().len();
        if script_pub_key_len > self.coinbase_payload_script_public_key_max_len as usize {
            return Err(CoinbaseError::PayloadScriptPublicKeyLenAboveMax(
                script_pub_key_len,
                self.coinbase_payload_script_public_key_max_len,
            ));
        }
        let payload: Vec<u8> = data.blue_score.to_le_bytes().iter().copied()                    // Blue score                   (u64)
            .chain(data.subsidy.to_le_bytes().iter().copied())                                  // Subsidy                      (u64)
            .chain(data.miner_data.script_public_key.version().to_le_bytes().iter().copied())   // Script public key version    (u16)
            .chain((script_pub_key_len as u8).to_le_bytes().iter().copied())                    // Script public key length     (u8)
            .chain(data.miner_data.script_public_key.script().iter().copied())                  // Script public key            
            .chain(data.miner_data.extra_data.as_ref().iter().copied())                         // Extra data
            .collect();

        Ok(payload)
    }

    pub fn modify_coinbase_payload<T: AsRef<[u8]>>(&self, mut payload: Vec<u8>, miner_data: &MinerData<T>) -> CoinbaseResult<Vec<u8>> {
        let script_pub_key_len = miner_data.script_public_key.script().len();
        if script_pub_key_len > self.coinbase_payload_script_public_key_max_len as usize {
            return Err(CoinbaseError::PayloadScriptPublicKeyLenAboveMax(
                script_pub_key_len,
                self.coinbase_payload_script_public_key_max_len,
            ));
        }

        // Keep only blue score and subsidy. Note that truncate does not modify capacity, so
        // the usual case where the payloads are the same size will not trigger a reallocation
        payload.truncate(LENGTH_OF_BLUE_SCORE + LENGTH_OF_SUBSIDY);
        payload.extend(
            miner_data.script_public_key.version().to_le_bytes().iter().copied() // Script public key version (u16)
                .chain((script_pub_key_len as u8).to_le_bytes().iter().copied()) // Script public key length  (u8)
                .chain(miner_data.script_public_key.script().iter().copied())    // Script public key
                .chain(miner_data.extra_data.as_ref().iter().copied()), // Extra data
        );

        Ok(payload)
    }

    pub fn deserialize_coinbase_payload<'a>(&self, payload: &'a [u8]) -> CoinbaseResult<CoinbaseData<&'a [u8]>> {
        if payload.len() < MIN_PAYLOAD_LENGTH {
            return Err(CoinbaseError::PayloadLenBelowMin(payload.len(), MIN_PAYLOAD_LENGTH));
        }

        if payload.len() > self.max_coinbase_payload_len {
            return Err(CoinbaseError::PayloadLenAboveMax(payload.len(), self.max_coinbase_payload_len));
        }

        let mut parser = PayloadParser::new(payload);

        let blue_score = u64::from_le_bytes(parser.take(LENGTH_OF_BLUE_SCORE).try_into().unwrap());
        let subsidy = u64::from_le_bytes(parser.take(LENGTH_OF_SUBSIDY).try_into().unwrap());
        let script_pub_key_version = u16::from_le_bytes(parser.take(LENGTH_OF_SCRIPT_PUB_KEY_VERSION).try_into().unwrap());
        let script_pub_key_len = u8::from_le_bytes(parser.take(LENGTH_OF_SCRIPT_PUB_KEY_LENGTH).try_into().unwrap());

        if script_pub_key_len > self.coinbase_payload_script_public_key_max_len {
            return Err(CoinbaseError::PayloadScriptPublicKeyLenAboveMax(
                script_pub_key_len as usize,
                self.coinbase_payload_script_public_key_max_len,
            ));
        }

        if parser.remaining.len() < script_pub_key_len as usize {
            return Err(CoinbaseError::PayloadCantContainScriptPublicKey(
                payload.len(),
                MIN_PAYLOAD_LENGTH + script_pub_key_len as usize,
            ));
        }

        let script_public_key =
            ScriptPublicKey::new(script_pub_key_version, ScriptVec::from_slice(parser.take(script_pub_key_len as usize)));
        let extra_data = parser.remaining;

        Ok(CoinbaseData { blue_score, subsidy, miner_data: MinerData { script_public_key, extra_data } })
    }

    pub fn calc_block_subsidy(&self, daa_score: u64) -> u64 {
        if daa_score < self.deflationary_phase_daa_score {
            return self.pre_deflationary_phase_base_subsidy;
        }

        let months_since_deflationary_phase_started =
            ((daa_score - self.deflationary_phase_daa_score) / self.blocks_per_month) as usize;
        if months_since_deflationary_phase_started >= self.subsidy_by_month_table.len() {
            *(self.subsidy_by_month_table).last().unwrap()
        } else {
            self.subsidy_by_month_table[months_since_deflationary_phase_started]
        }
    }

    #[cfg(test)]
    pub fn legacy_calc_block_subsidy(&self, daa_score: u64) -> u64 {
        if daa_score < self.deflationary_phase_daa_score {
            return self.pre_deflationary_phase_base_subsidy;
        }

        // Note that this calculation implicitly assumes that block per second = 1 (by assuming daa score diff is in second units).
        let months_since_deflationary_phase_started = (daa_score - self.deflationary_phase_daa_score) / SECONDS_PER_MONTH;
        assert!(months_since_deflationary_phase_started <= usize::MAX as u64);
        let months_since_deflationary_phase_started: usize = months_since_deflationary_phase_started as usize;
        if months_since_deflationary_phase_started >= SUBSIDY_BY_MONTH_TABLE.len() {
            *SUBSIDY_BY_MONTH_TABLE.last().unwrap()
        } else {
            SUBSIDY_BY_MONTH_TABLE[months_since_deflationary_phase_started]
        }
    }
}

/*
    This table was pre-calculated by calling `calcDeflationaryPeriodBlockSubsidyFloatCalc` (in spectred-go) for all months until reaching 0 subsidy.
    To regenerate this table, run `TestBuildSubsidyTable` in coinbasemanager_test.go (note the `deflationaryPhaseBaseSubsidy` therein).
    These values apply to 1 block per second.
*/
#[rustfmt::skip]
const SUBSIDY_BY_MONTH_TABLE: [u64; 727] = [
	1200000000, 1175000000, 1150000000, 1125000000, 1100000000, 1075000000, 1050000000, 1025000000, 1000000000, 975000000, 950000000, 925000000, 900000000, 875000000, 850000000, 825000000, 800000000, 775000000, 750000000, 725000000, 700000000, 675000000, 650000000, 625000000, 600000000,
	587500000, 575000000, 562500000, 550000000, 537500000, 525000000, 512500000, 500000000, 487500000, 475000000, 462500000, 450000000, 437500000, 425000000, 412500000, 400000000, 387500000, 375000000, 362500000, 350000000, 337500000, 325000000, 312500000, 300000000, 293750000,
	287500000, 281250000, 275000000, 268750000, 262500000, 256250000, 250000000, 243750000, 237500000, 231250000, 225000000, 218750000, 212500000, 206250000, 200000000, 193750000, 187500000, 181250000, 175000000, 168750000, 162500000, 156250000, 150000000, 146875000, 143750000,
	140625000, 137500000, 134375000, 131250000, 128125000, 125000000, 121875000, 118750000, 115625000, 112500000, 109375000, 106250000, 103125000, 100000000, 96875000, 93750000, 90625000, 87500000, 84375000, 81250000, 78125000, 75000000, 73437500, 71875000, 70312500,
	68750000, 67187500, 65625000, 64062500, 62500000, 60937500, 59375000, 57812500, 56250000, 54687500, 53125000, 51562500, 50000000, 48437500, 46875000, 45312500, 43750000, 42187500, 40625000, 39062500, 37500000, 36718750, 35937500, 35156250, 34375000,
	33593750, 32812500, 32031250, 31250000, 30468750, 29687500, 28906250, 28125000, 27343750, 26562500, 25781250, 25000000, 24218750, 23437500, 22656250, 21875000, 21093750, 20312500, 19531250, 18750000, 18359375, 17968750, 17578125, 17187500, 16796875,
	16406250, 16015625, 15625000, 15234375, 14843750, 14453125, 14062500, 13671875, 13281250, 12890625, 12500000, 12109375, 11718750, 11328125, 10937500, 10546875, 10156250, 9765625, 9375000, 9179687, 8984375, 8789062, 8593750, 8398437, 8203125,
	8007812, 7812500, 7617187, 7421875, 7226562, 7031250, 6835937, 6640625, 6445312, 6250000, 6054687, 5859375, 5664062, 5468750, 5273437, 5078125, 4882812, 4687500, 4589843, 4492187, 4394531, 4296875, 4199218, 4101562, 4003906,
	3906250, 3808593, 3710937, 3613281, 3515625, 3417968, 3320312, 3222656, 3125000, 3027343, 2929687, 2832031, 2734375, 2636718, 2539062, 2441406, 2343750, 2294921, 2246093, 2197265, 2148437, 2099609, 2050781, 2001953, 1953125,
	1904296, 1855468, 1806640, 1757812, 1708984, 1660156, 1611328, 1562500, 1513671, 1464843, 1416015, 1367187, 1318359, 1269531, 1220703, 1171875, 1147460, 1123046, 1098632, 1074218, 1049804, 1025390, 1000976, 976562, 952148,
	927734, 903320, 878906, 854492, 830078, 805664, 781250, 756835, 732421, 708007, 683593, 659179, 634765, 610351, 585937, 573730, 561523, 549316, 537109, 524902, 512695, 500488, 488281, 476074, 463867,
	451660, 439453, 427246, 415039, 402832, 390625, 378417, 366210, 354003, 341796, 329589, 317382, 305175, 292968, 286865, 280761, 274658, 268554, 262451, 256347, 250244, 244140, 238037, 231933, 225830,
	219726, 213623, 207519, 201416, 195312, 189208, 183105, 177001, 170898, 164794, 158691, 152587, 146484, 143432, 140380, 137329, 134277, 131225, 128173, 125122, 122070, 119018, 115966, 112915, 109863,
	106811, 103759, 100708, 97656, 94604, 91552, 88500, 85449, 82397, 79345, 76293, 73242, 71716, 70190, 68664, 67138, 65612, 64086, 62561, 61035, 59509, 57983, 56457, 54931, 53405,
	51879, 50354, 48828, 47302, 45776, 44250, 42724, 41198, 39672, 38146, 36621, 35858, 35095, 34332, 33569, 32806, 32043, 31280, 30517, 29754, 28991, 28228, 27465, 26702, 25939,
	25177, 24414, 23651, 22888, 22125, 21362, 20599, 19836, 19073, 18310, 17929, 17547, 17166, 16784, 16403, 16021, 15640, 15258, 14877, 14495, 14114, 13732, 13351, 12969, 12588,
	12207, 11825, 11444, 11062, 10681, 10299, 9918, 9536, 9155, 8964, 8773, 8583, 8392, 8201, 8010, 7820, 7629, 7438, 7247, 7057, 6866, 6675, 6484, 6294, 6103,
	5912, 5722, 5531, 5340, 5149, 4959, 4768, 4577, 4482, 4386, 4291, 4196, 4100, 4005, 3910, 3814, 3719, 3623, 3528, 3433, 3337, 3242, 3147, 3051, 2956,
	2861, 2765, 2670, 2574, 2479, 2384, 2288, 2241, 2193, 2145, 2098, 2050, 2002, 1955, 1907, 1859, 1811, 1764, 1716, 1668, 1621, 1573, 1525, 1478, 1430,
	1382, 1335, 1287, 1239, 1192, 1144, 1120, 1096, 1072, 1049, 1025, 1001, 977, 953, 929, 905, 882, 858, 834, 810, 786, 762, 739, 715, 691,
	667, 643, 619, 596, 572, 560, 548, 536, 524, 512, 500, 488, 476, 464, 452, 441, 429, 417, 405, 393, 381, 369, 357, 345, 333,
	321, 309, 298, 286, 280, 274, 268, 262, 256, 250, 244, 238, 232, 226, 220, 214, 208, 202, 196, 190, 184, 178, 172, 166, 160,
	154, 149, 143, 140, 137, 134, 131, 128, 125, 122, 119, 116, 113, 110, 107, 104, 101, 98, 95, 92, 89, 86, 83, 80, 77,
	74, 71, 70, 68, 67, 65, 64, 62, 61, 59, 58, 56, 55, 53, 52, 50, 49, 47, 46, 44, 43, 41, 40, 38, 37,
	35, 35, 34, 33, 32, 32, 31, 30, 29, 29, 28, 27, 26, 26, 25, 24, 23, 23, 22, 21, 20, 20, 19, 18, 17,
	17, 17, 16, 16, 16, 15, 15, 14, 14, 14, 13, 13, 13, 12, 12, 11, 11, 11, 10, 10, 10, 9, 9, 8, 8,
	8, 8, 8, 8, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4,
	4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
	2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
	1, 0,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::MAINNET_PARAMS;
    use spectre_consensus_core::{
        config::params::{Params, TESTNET11_PARAMS},
        constants::SOMPI_PER_SPECTRE,
        network::NetworkId,
        tx::scriptvec,
    };

    #[test]
    fn calc_high_bps_total_rewards_delta() {
        const SECONDS_PER_MONTH: u64 = 2629800;

        let legacy_cbm = create_legacy_manager();
        let pre_deflationary_rewards = legacy_cbm.pre_deflationary_phase_base_subsidy * legacy_cbm.deflationary_phase_daa_score;
        let total_rewards: u64 = pre_deflationary_rewards + SUBSIDY_BY_MONTH_TABLE.iter().map(|x| x * SECONDS_PER_MONTH).sum::<u64>();
        let testnet_11_bps = TESTNET11_PARAMS.bps();
        let total_high_bps_rewards_rounded_up: u64 = pre_deflationary_rewards
            + SUBSIDY_BY_MONTH_TABLE
                .iter()
                .map(|x| ((x + testnet_11_bps - 1) / testnet_11_bps * testnet_11_bps) * SECONDS_PER_MONTH)
                .sum::<u64>();

        let cbm = create_manager(&TESTNET11_PARAMS);
        let total_high_bps_rewards: u64 =
            pre_deflationary_rewards + cbm.subsidy_by_month_table.iter().map(|x| x * cbm.blocks_per_month).sum::<u64>();
        assert_eq!(total_high_bps_rewards_rounded_up, total_high_bps_rewards, "subsidy adjusted to bps must be rounded up");

        let delta = total_high_bps_rewards as i64 - total_rewards as i64;

        println!("Total rewards: {} sompi => {} SPR", total_rewards, total_rewards / SOMPI_PER_SPECTRE);
        println!("Total high bps rewards: {} sompi => {} SPR", total_high_bps_rewards, total_high_bps_rewards / SOMPI_PER_SPECTRE);
        println!("Delta: {} sompi => {} SPR", delta, delta / SOMPI_PER_SPECTRE as i64);
    }

    #[test]
    fn subsidy_by_month_table_test() {
        let cbm = create_legacy_manager();
        cbm.subsidy_by_month_table.iter().enumerate().for_each(|(i, x)| {
            assert_eq!(SUBSIDY_BY_MONTH_TABLE[i], *x, "for 1 BPS, const table and precomputed values must match");
        });

        for network_id in NetworkId::iter() {
            let cbm = create_manager(&network_id.into());
            cbm.subsidy_by_month_table.iter().enumerate().for_each(|(i, x)| {
                assert_eq!(
                    (SUBSIDY_BY_MONTH_TABLE[i] + cbm.bps() - 1) / cbm.bps(),
                    *x,
                    "{}: locally computed and precomputed values must match",
                    network_id
                );
            });
        }
    }

    #[test]
    fn subsidy_test() {
        const PRE_DEFLATIONARY_PHASE_BASE_SUBSIDY: u64 = 1500000000;
        const DEFLATIONARY_PHASE_INITIAL_SUBSIDY: u64 = 1200000000;
        const SECONDS_PER_MONTH: u64 = 2629800;
        const SECONDS_PER_HALVING: u64 = SECONDS_PER_MONTH * 24;

        for network_id in NetworkId::iter() {
            let params = &network_id.into();
            let cbm = create_manager(params);

            let pre_deflationary_phase_base_subsidy = PRE_DEFLATIONARY_PHASE_BASE_SUBSIDY / params.bps();
            let deflationary_phase_initial_subsidy = DEFLATIONARY_PHASE_INITIAL_SUBSIDY / params.bps();
            let blocks_per_halving = SECONDS_PER_HALVING * params.bps();

            struct Test {
                name: &'static str,
                daa_score: u64,
                expected: u64,
            }

            let tests = vec![
                Test { name: "first mined block", daa_score: 1, expected: pre_deflationary_phase_base_subsidy },
                Test {
                    name: "before deflationary phase",
                    daa_score: params.deflationary_phase_daa_score - 1,
                    expected: pre_deflationary_phase_base_subsidy,
                },
                Test {
                    name: "start of deflationary phase",
                    daa_score: params.deflationary_phase_daa_score,
                    expected: deflationary_phase_initial_subsidy,
                },
                Test {
                    name: "after 2 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving,
                    expected: deflationary_phase_initial_subsidy / 2,
                },
                Test {
                    name: "after 4 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 2,
                    expected: deflationary_phase_initial_subsidy / 4,
                },
                Test {
                    name: "after 8 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 4,
                    expected: deflationary_phase_initial_subsidy / 16,
                },
                Test {
                    name: "after 16 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 8,
                    expected: deflationary_phase_initial_subsidy / 256,
                },
                Test {
                    name: "after 32 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 16,
                    expected: deflationary_phase_initial_subsidy / 65536,
                },
                Test {
                    name: "just before subsidy depleted",
                    daa_score: params.deflationary_phase_daa_score + (blocks_per_halving / 24 * 725),
                    expected: 1,
                },
                Test {
                    name: "after subsidy depleted",
                    daa_score: params.deflationary_phase_daa_score + (blocks_per_halving / 24 * 726),
                    expected: 0,
                },
            ];

            for t in tests {
                assert_eq!(cbm.calc_block_subsidy(t.daa_score), t.expected, "{} test '{}' failed", network_id, t.name);
                if params.bps() == 1 {
                    assert_eq!(cbm.legacy_calc_block_subsidy(t.daa_score), t.expected, "{} test '{}' failed", network_id, t.name);
                }
            }
        }
    }

    #[test]
    fn payload_serialization_test() {
        let cbm = create_manager(&MAINNET_PARAMS);

        let script_data = [33u8, 255];
        let extra_data = [2u8, 3];
        let data = CoinbaseData {
            blue_score: 56,
            subsidy: 1200000000,
            miner_data: MinerData {
                script_public_key: ScriptPublicKey::new(0, ScriptVec::from_slice(&script_data)),
                extra_data: &extra_data as &[u8],
            },
        };

        let payload = cbm.serialize_coinbase_payload(&data).unwrap();
        let deserialized_data = cbm.deserialize_coinbase_payload(&payload).unwrap();

        assert_eq!(data, deserialized_data);

        // Test an actual mainnet payload
        let payload_hex =
            "b612c90100000000041a763e07000000000022202b32443ff740012157716d81216d09aebc39e5493c93a7181d92cb756c02c560ac302e31322e382f";
        let mut payload = vec![0u8; payload_hex.len() / 2];
        faster_hex::hex_decode(payload_hex.as_bytes(), &mut payload).unwrap();
        let deserialized_data = cbm.deserialize_coinbase_payload(&payload).unwrap();

        let expected_data = CoinbaseData {
            blue_score: 29954742,
            subsidy: 31112698372,
            miner_data: MinerData {
                script_public_key: ScriptPublicKey::new(
                    0,
                    scriptvec![
                        32, 43, 50, 68, 63, 247, 64, 1, 33, 87, 113, 109, 129, 33, 109, 9, 174, 188, 57, 229, 73, 60, 147, 167, 24,
                        29, 146, 203, 117, 108, 2, 197, 96, 172,
                    ],
                ),
                extra_data: &[48u8, 46, 49, 50, 46, 56, 47] as &[u8],
            },
        };
        assert_eq!(expected_data, deserialized_data);
    }

    #[test]
    fn modify_payload_test() {
        let cbm = create_manager(&MAINNET_PARAMS);

        let script_data = [33u8, 255];
        let extra_data = [2u8, 3, 23, 98];
        let data = CoinbaseData {
            blue_score: 56345,
            subsidy: 1200000000,
            miner_data: MinerData {
                script_public_key: ScriptPublicKey::new(0, ScriptVec::from_slice(&script_data)),
                extra_data: &extra_data,
            },
        };

        let data2 = CoinbaseData {
            blue_score: data.blue_score,
            subsidy: data.subsidy,
            miner_data: MinerData {
                // Modify only miner data
                script_public_key: ScriptPublicKey::new(0, ScriptVec::from_slice(&[33u8, 255, 33])),
                extra_data: &[2u8, 3, 23, 98, 34, 34] as &[u8],
            },
        };

        let mut payload = cbm.serialize_coinbase_payload(&data).unwrap();
        payload = cbm.modify_coinbase_payload(payload, &data2.miner_data).unwrap(); // Update the payload with the modified miner data
        let deserialized_data = cbm.deserialize_coinbase_payload(&payload).unwrap();

        assert_eq!(data2, deserialized_data);
    }

    fn create_manager(params: &Params) -> CoinbaseManager {
        CoinbaseManager::new(
            params.coinbase_payload_script_public_key_max_len,
            params.max_coinbase_payload_len,
            params.deflationary_phase_daa_score,
            params.pre_deflationary_phase_base_subsidy,
            params.target_time_per_block,
        )
    }

    /// Return a CoinbaseManager with legacy golang 1 BPS properties
    fn create_legacy_manager() -> CoinbaseManager {
        CoinbaseManager::new(150, 204, 604800, 1500000000, 1000)
    }
}
