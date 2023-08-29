Gaia-rs is a [Cosmos Hub](https://github.com/cosmos/gaia) (Gaia) node built on Gears, a Rust implementation of the Cosmos SDK.

NOTE: This is a WIP. Although it correctly replicates state for a subset of v0.7.1.0 of the Gaia state machine, the full state machine has not yet been implemented.

# Getting Started

Gears uses the [tendermint-abci](https://crates.io/crates/tendermint-abci) crate to communicate with a Tendermint instance which runs as a separate process. This means that to run a full node, Tendermint must be installed and run separately (see instructions below).

## Requirements

**Rust compiler**

The minimum supported Rust version is 1.67.1. Follow the [installation instructions](https://doc.rust-lang.org/book/ch01-01-installation.html).

**Tendermint**

Gaia v0.7.1.0 uses Tendermint version v0.34.21. After cloning the [Tendermint repo](https://github.com/tendermint/tendermint) checkout v0.34.21 then follow the [installation instructions](https://github.com/tendermint/tendermint/blob/v0.34.21/docs/introduction/install.md).

**libclang**

This is needed by the rocks db crate, run `sudo apt install libclang-dev build-essential`.

## Running a local chain

1. Clone this repo:

```console
git clone https://github.com/rumos-io/gears
cd gears
```

2. Initialize a new chain:

```console
make init
```

3. Build and start the application:

```console
make run
```

The application will listen for connections on tcp://127.0.0.1:26658.

4. From a different terminal window start Tendermint:

```console
make tendermint-start
```

Tendermint will connect to the application and bind it's RPC server to 127.0.0.1:26657.

The chain (consisting of one node) is now up and running.


## Querying the chain

So far we've been running gaia-rs indirectly using make commands and the rust build tool, Cargo. In this
section we'll install gaia-rs and use it to query the chain (just like cosmos-sdk based chains the gaia-rs binary serves as a
node and client).

1. Install gaia-rs:

```console
make install
```

2. Query a balance:

```console
gaia-rs query bank balances cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux
```

Which returns:

```json
{
  "balances": [
    {
      "denom": "uatom",
      "amount": "34"
    }
  ],
  "pagination": null
}
```

The balance of this address was set to 34 in the genesis file.

3. Import the key corresponding to the above address into the gaia-rs key store:

```console
echo "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow" | gaia-rs keys add alice --recover
```

4. Send tokens:

```console
gaia-rs tx alice bank send cosmos180tr8wmsk8ugt32yynj8efqwg3yglmpwp22rut 10uatom --fee 1uatom
```

5. Query the address balance and observe that it has decreased by 11uatom which is the sum of the amount transferred and the fee:

```console
gaia-rs query bank balances cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux
```

Which returns:

```json
{
  "balances": [
    {
      "denom": "uatom",
      "amount": "23"
    }
  ],
  "pagination": null
}

```