# Gaia-rs

Gaia-rs will be a cosmos hub (gaia) node written in Rust. It will initially implement v7.1.0 of the gaia state machine as specified by the [golang implementation](https://github.com/cosmos/gaia/tree/v7.1.0).

NOTE: this is a WIP. As it stands not all of the gaia state machine has been implemented so it isn't able to correctly replicate state.

# Getting Started

Gaia-rs uses [tendermint-abci](https://crates.io/crates/tendermint-abci) to communicate with a Tendermint instance which runs as a separate process. So to run a node, Tendermint must be installed and run separately (see instructions below).
## Requirements

Rust compiler - The minimum supported Rust version is 1.67.1 - [Installation instructions](https://doc.rust-lang.org/book/ch01-01-installation.html).

Tendermint - Gaia v0.7.1.0 uses Tendermint version v0.34.21 - Follow the [installation instructions](https://github.com/tendermint/tendermint/blob/main/docs/introduction/install.md) ensuring to checkout v0.34.21.

Gaiad - Currently gaia-rs does not implement a client so we use the golang implementation. To install gaiad, clone the [gaia repo](https://github.com/cosmos/gaia), checkout `v7.1.0` then run `make install`.

## Running

1. From the gaia-rs directory change into the app directory then run make to build and start the application:

```
cd app
make run
```

The application will listen for connections on tcp://127.0.0.1:26658

2. Start Tendermint:

```
make tendermint-clean-start
```

Tendermint will connect to the application and will bind the RPC server to 127.0.0.1:26657


3. Query a balance:

```
gaiad query bank balances cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux
```

Which returns:

```yaml
balances:
- amount: "34"
  denom: uatom
pagination: null
```

The balance is set at 34 in a hard coded genesis structure.

4. Import the key corresponding to the above address into gaiad:

```
echo "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow" | gaiad keys add demo --recover --keyring-backend=test
```

5. Send tokens

```
gaiad tx bank send cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut 10uatom --keyring-backend=test --chain-id=localnet --broadcast-mode block --fees 1uatom
```

6. Query the address balance and observe that it has decreased by 11uatom ( = 10uatom + fee):

```
gaiad query bank balances cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux
```

Which returns:

```yaml
balances:
- amount: "23"
  denom: uatom
pagination: null
```
