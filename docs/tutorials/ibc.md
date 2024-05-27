# IBC

This is more of a cheat sheet than a tutorial at this stage.

## Creating an ibc client

```shell
gaiad tx ibc client create ibc_config/tendermint_client_state.json ibc_config/tendermint_consensus_state.json --from bob --keyring-backend=test --chain-id=test-chain
```

Which gives the following app hash in the go implementation of gaia (this isn't deterministic):

app_hash=????

```shell
gaia-rs tx local alice ibc client create ibc_config/tendermint_client_state.json ibc_config/tendermint_consensus_state.json
```

```shell
cargo run -- tx local alice ibc client create docs/tutorials/ibc_config/gaia_rs_client_state.json docs/tutorials/ibc_config/gaia_rs_consensus_state.json
```

Check if it was created:

```shell
gaiad query ibc client states --output=json
```

```shell
cargo run -- query ibc client states
```

## Relaying with hermes

1. Add chain data to hermes config in `~/.hermes/config.toml`:

```toml
[[chains]]
type = "CosmosSdk"
id = 'test-chain'
rpc_addr = 'http://localhost:26657'
grpc_addr = 'tcp://localhost:8080'
#websocket_addr = 'ws://localhost:26557/websocket'
rpc_timeout = '10s'
account_prefix = 'cosmos'
key_name = 'testkey'
store_prefix = 'ibc'
max_gas = 2000000
gas_price = { price = 0.001, denom = 'uatom' }
gas_multiplier = 1.1
clock_drift = '5s'
trusting_period = '14days'
trust_threshold = { numerator = '1', denominator = '3' }

[chains.event_source]
mode = "push"
url = "ws://localhost:26557/websocket"
batch_delay = "500ms"

[[chains]]
type = "CosmosSdk"
id = 'test-chain-1'
rpc_addr = 'http://localhost:26660'
grpc_addr = 'tcp://localhost:8080'
#websocket_addr = 'ws://localhost:26557/websocket'
rpc_timeout = '10s'
account_prefix = 'cosmos'
key_name = 'testkey'
store_prefix = 'ibc'
max_gas = 2000000
gas_price = { price = 0.001, denom = 'uatom' }
gas_multiplier = 1.1
clock_drift = '5s'
trusting_period = '14days'
trust_threshold = { numerator = '1', denominator = '3' }

[chains.event_source]
mode = "push"
url = "ws://localhost:26557/websocket"
batch_delay = "500ms"
```

2. Query packets

```shell
hermes query packet unreceived-packets test-chain transfer channel-7
```

```shell
hermes query packet acks test-chain transfer channel-7
```

response (failed):

```txt
Got query request to: store/ibc/key
```

### Opening a connection

This requires keys:

```shell
echo "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow" | hermes keys add --chain test-chain --mnemonic-file /dev/stdin
```

```shell
echo "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow" | hermes keys add --chain test-chain-1 --mnemonic-file /dev/stdin
```

```shell
hermes create connection --a-chain test-chain --b-chain test-chain-1
```

## Getting client state and consensus state

Get client states:

```shell
gaiad query ibc client states --node https://cosmos-rpc.publicnode.com:443
```

Get consensus states (`07-tendermint-1000` was found to return non empty consensus states):

```shell
gaiad query ibc client consensus-states 07-tendermint-1000 --node https://cosmos-rpc.publicnode.com:443 --output=json
```
