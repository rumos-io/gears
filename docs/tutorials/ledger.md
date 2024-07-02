# Ledger tutorial

1. Start a chain as in the `gaia-rs` README.

2. Send funds to ledger address - for now you'll need to use a cosmos based chain's CLI to get the address!

```shell
gaia-rs tx --keyring local --from-key alice bank send cosmos12vrgunwvszgzpykdrqlx3m6puedvcajlxcyw8z 30uatom
```

3. Check balance of ledger address

```shell
gaia-rs query bank balances cosmos12vrgunwvszgzpykdrqlx3m6puedvcajlxcyw8z
```

4. Send from ledger

```shell
gaia-rs tx --keyring ledger bank send cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux 1uatom
```
