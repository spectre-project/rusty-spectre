### Latest Release

- Support numeric interface (IP) argument without port in `--rpclisten-borsh` or `--rpclisten-json`
- Replace `MassCalculator` with `calculateTransactionMass` and `calculateTransactionFee` functions.
- Change `createTransaction` function signature (remove requirement for change address).
- Make `ITransactionInput.signatureScript` optional (if not supplied, the signatureScript is assigned an empty vector).

- Fix issues with deserializing manually-created objects matching `IUtxoEntry` interface.
- Allow arguments expecting ScriptPublicKey to receive `{ version, script }` object or a hex string.
- Fix `Transaction::serializeToObject()` return type (now returning `ISerializeTransaction` interface).
- Adding `setUserTransactionMaturityDAA()` and `setCoinbaseTransactionMaturityDAA()` that allow customizing
  the maturity DAA periods for user and coinbase transactions.

- Fix `PublicKeyGenerator::change_address_as_string()` that was returning the receive address.
- WASM SDK now builds as a GitHub artifact during the CI process.
- `State` renamed to `PoW`
- Docs now have a PoW section that unifies all PoW-related classes and functions.
- `TransactionRecord.data` (`TransactionData`) now has correct TypeScript bindings.

### Release 2024-05-24

- First version with Spectre Network support
