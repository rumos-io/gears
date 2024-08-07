1. Query the bank balances by denom endpoint

```shell
curl localhost:1317/cosmos/bank/v1beta1/balances/cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux/by_denom?denom=uatom | jq
```

2. Query transactions

```shell
curl localhost:1317/cosmos/tx/v1beta1/txs?events=transfer.sender=\'cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux\'
```

```shell
curl localhost:1317/cosmos/tx/v1beta1/txs?events=transfer.recipient=\'cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux\'
```

3. Query supply

```shell
curl localhost:1317/cosmos/bank/v1beta1/supply
```
