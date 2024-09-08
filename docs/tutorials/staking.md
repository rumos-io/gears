# Staking tutorial

## Adding validators

1. Create a validator

```shell
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

```shell
gaiad query staking validators
```

## Genesis validator

```shell
gaia-rs gentx 10uatom --from-key alice --account-number 2 --sequence 0 --keyring-backend=test
```

```shell
gaia-rs collect-gentxs
```
