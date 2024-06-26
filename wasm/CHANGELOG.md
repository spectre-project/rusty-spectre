### Latest Release

- Fix issues with deserializing manually-created objects matching `IUtxoEntry` interface.
- Allow arguments expecting ScriptPublicKey to receive `{ version, script }` object or a hex string.
- Fix `Transaction::serializeToObject()` return type (now returning `ISerializeTransaction` interface).
- Adding `setUserTransactionMaturityDAA()` and `setCoinbaseTransactionMaturityDAA()` that allow customizing
the maturity DAA periods for user and coinbase transactions.

### Release 2024-05-24

- First version with Spectre Network support

### Release 2024-06-21

- Fix `PublicKeyGenerator::change_address_as_string()` that was returning the receive address.
- WASM SDK now builds as a GitHub artifact during the CI process.
- `State` renamed to `PoW`
- Docs now have a PoW section that unifies all PoW-related classes and functions.
- `TransactionRecord.data` (`TransactionData`) now has correct TypeScript bindings.
- Adding utility functions:  `payToAddressScript`, `payToScriptHashScript`, `payToScriptHashSignatureScript`, `addressFromScriptPublicKey`, `isScriptPayToPubkey`, `isScriptPayToPubkeyECDSA`, `isScriptPayToScriptHash`.
- Adding `UtxoProcessor::isActive` property to check if the processor is in active state (connected and running). This property can be used to validate the processor state before invoking it's functions (that can throw is the UtxoProcessor is offline).
- Rename `UtxoContext::active` to `UtxoContext::isActive` for consistency.
