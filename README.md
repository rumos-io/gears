# ABCI Demo App

As it stands this is simply a copy of the key/value store example given in the [tendermint-rs repo](https://github.com/informalsystems/tendermint-rs/tree/main/abci).

# Requirements

Rust compiler - Tested with version 1.63.0 - [Installation instructions](https://doc.rust-lang.org/book/ch01-01-installation.html)

Tendermint - Tested with v0.35.0 - [Installation instructions](https://github.com/tendermint/tendermint/blob/main/docs/introduction/install.md)

# Running the app

1. Start the application:

```
make run
```

The application will listen for connections on tcp://127.0.0.1:26658

2. Start Tendermint:

```
make tendermint-clean-start
```

Tendermint will connect to the application and will bind the RPC server to 127.0.0.1:26657


3. Set a key/value pair in the keystore (set "somekey" to "somevalue"):

```
curl 'http://127.0.0.1:26657/broadcast_tx_async?tx="somekey=somevalue"'
```

4. Query the store for the value of "somekey" ("736f6d656b6579" is the hex representation of "somekey"):

```
curl 'http://127.0.0.1:26657/abci_query?data=0x736f6d656b6579'
```

Which will return:

```
{
  "jsonrpc": "2.0",
  "id": -1,
  "result": {
    "response": {
      "code": 0,
      "log": "exists",
      "info": "",
      "index": "0",
      "key": "c29tZWtleQ==",
      "value": "c29tZXZhbHVl",
      "proofOps": null,
      "height": "14",
      "codespace": ""
    }
  }
}
```
