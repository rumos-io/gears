# Staking tutorial

1. Create a validator

```
gaiad tx staking create-validator \
  --amount=5uatom \
  --pubkey='{"@type":"/cosmos.crypto.ed25519.PubKey","key":"+uo5x4+nFiCBt2MuhVwT5XeMfj6ttkjY/JC6WyHb+rE="}' \
  --moniker="my_val" \
  --chain-id=test-chain \
  --commission-rate="0.10" \
  --commission-max-rate="0.20" \
  --commission-max-change-rate="0.1" \
  --min-self-delegation="1" \
  --from=bob \
  --keyring-backend=test \
  --broadcast-mode block \
  -y
```

```
gaiad query staking validators
```
