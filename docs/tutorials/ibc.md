# Relaying with hermes

1. Add chain data to hermes config in `~/.hermes/config.toml`:

```toml
[[chains]]
id = 'test-chain'
rpc_addr = 'http://localhost:26657'
grpc_addr = 'http://localhost:9090'
websocket_addr = 'ws://localhost:26557/websocket'
rpc_timeout = '10s'
account_prefix = 'atom'
key_name = 'testkey'
store_prefix = 'ibc'
max_gas = 2000000
gas_price = { price = 0.001, denom = 'stake' }
gas_adjustment = 0.1
clock_drift = '5s'
trusting_period = '14days'
trust_threshold = { numerator = '1', denominator = '3' }
```

2.

```shell
hermes query packet unreceived-packets test-chain transfer channel-7
```

```shell
hermes query packet acks test-chain transfer channel-7
```

response:

```txt
Got query request to: store/ibc/key
```
