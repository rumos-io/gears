# Querying the gRPC endpoints

## List of services

```shell
grpcurl -plaintext localhost:8080 list
```

## No arguments

```shell
grpcurl -plaintext  -d '{}' localhost:8080  cosmos.staking.v1beta1.Query/Params
```

```json
{
  "params": {
    "unbondingTime": "1814400s",
    "maxValidators": 12,
    "maxEntries": 100,
    "historicalEntries": 10,
    "bondDenom": "uatom",
    "minCommissionRate": "0.1"
  }
}
```

## With arguments

```shell
grpcurl -plaintext  -d '{"address": "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux", "denom":"uatom"}' localhost:8080  cosmos.bank.v1beta1.Query/Balance
```
