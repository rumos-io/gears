# Querying the gRPC endpoints

```shell
grpcurl -plaintext localhost:8080 list
```

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
