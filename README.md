# balanced-rust-contracts
[![Rust Cov][rust-cov-badge]][rust-cov-link]

[rust-cov-link]: https://app.codecov.io/gh/balancednetwork/balanced-rust-contracts/tree/main/contracts
[rust-cov-badge]: https://codecov.io/gh/balancednetwork/balanced-rust-contracts/branch/main/graph/badge.svg?flag=rust

Rust contracts for Balanced xCall integrations
## General Build
To build the contract and verify built wasm file run following script:

`./scripts/generate_wasm.sh`


## Chain Specific Build
To build the contract for specific chain with its own features, enable the feature as default by modifying `Cargo.toml`.

### Chain Injective
For injective chain we enable feature `injective` in all projects and subprojects by modifying feature section as follows:
```
[features]
default=["injective"]
```
#### Additional Steps Injective
- Call method `SetAdapter` on this contract to set adapter contract.
```
injectived tx wasm execute ${TOKEN_CONTRACT} '{"set_adapter":{"registry_contract":"${REGISTRY_CONTRACT}"}}' --from ${WALLET} --keyring-backend test --node https://injective-testnet-rpc.publicnode.com:443 --chain-id injective-888 --gas-prices 500000000inj --gas auto --gas-adjustment 1.5 -y --output json
```
- Register this contract to CW20Adapter contract by calling  `RegisterCW20` on adapter contract.
```
injectived tx wasm execute ${REGISTRY_CONTRACT} '{"register_cw20_contract":{"addr":"${TOKEN_CONTRACT}"}}' --from ${WALLET} --keyring-backend test --amount 1000000000000000000inj --node https://injective-testnet-rpc.publicnode.com:443 --chain-id injective-888 --gas-prices 500000000inj --gas auto --gas-adjustment 1.5 -y --output json
```
- Update metadata on CW20Adapter by calling `UpdateMetadata`.
```
injectived tx wasm execute ${REGISTRY_CONTRACT} '{"update_metadata":{"addr":"${TOKEN_CONTRACT}"}}' --from ${WALLET} --keyring-backend test --amount 1000000000000000000inj --node https://injective-testnet-rpc.publicnode.com:443 --chain-id injective-888 --gas-prices 500000000inj --gas auto --gas-adjustment 1.5 -y --output json

```
After properly setup we should be able to query balance transferred to user as denom token balance as follows:
```
injectived query bank spendable-balance ${USER_ACCOUNT} factory/${REGISTRY_CONTRACT}/${TOKEN_CONTRACT} --node https://injective-testnet-rpc.publicnode.com:443 --chain-id injective-888
```




