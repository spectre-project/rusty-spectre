//! PSST roles.

/// Initializes the PSST with 0 inputs and 0 outputs.
/// Reference: [BIP-370: Creator](https://github.com/bitcoin/bips/blob/master/bip-0370.mediawiki#creator)
pub enum Creator {}

/// Adds inputs and outputs to the PSST.
/// Reference: [BIP-370: Constructor](https://github.com/bitcoin/bips/blob/master/bip-0370.mediawiki#constructor)
pub enum Constructor {}

/// Can set the sequence number.
/// Reference: [BIP-370: Updater](https://github.com/bitcoin/bips/blob/master/bip-0370.mediawiki#updater)
pub enum Updater {}

/// Creates cryptographic signatures for the inputs using private keys.
/// Reference: [BIP-370: Signer](https://github.com/bitcoin/bips/blob/master/bip-0370.mediawiki#signer)
pub enum Signer {}

/// Merges multiple PSSTs into one.
/// Reference: [BIP-174: Combiner](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki#combiner)
pub enum Combiner {}

/// Completes the PSST, ensuring all inputs have valid signatures, and finalizes the transaction.
/// Reference: [BIP-174: Input Finalizer](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki#input-finalizer)
pub enum Finalizer {}

/// Extracts the final transaction from the PSST once all parts are in place and the PSST is fully signed.
/// Reference: [BIP-370: Transaction Extractor](https://github.com/bitcoin/bips/blob/master/bip-0370.mediawiki#transaction-extractor)
pub enum Extractor {}
