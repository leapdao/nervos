# Bridge Client

## Test run

First launch a local CKB node and a miner. Also make sure your ckb-cli is pointed and synced with this local network. Also make sure the miner address is set to one of your ckb-cli accoutns, so you will have some coins.

Next, run `capsule deploy --address {miner_address}` from the parent-bridge directory in the repo. This will deploy the code for our 2 scripts, and print some data you will later need for configuration, so keep it close.

Now open the test.ts file and replace myConfig with the following (fill in missing values):
```
{
  SIGHASH_DEP: {
    out_point: {
      tx_hash: "0xace5ea83c478bb866edf122ff862085789158f5cbff155b7bb5f13058555b708",
      index: "0x0",
    },
    dep_type: "dep_group",
  },
  BRIDGE_DEP: {
    out_point: {
      tx_hash: <take from capsule deploy logs>,
      index: <take from capsule deploy logs>,
    },
    dep_type: "code",
  },
  DEPOSIT_DEP: {
    out_point: {
      tx_hash: <take from capsule deploy logs>,
      index: <take from capsule deploy logs>,
    },
    dep_type: "code",
  },
  ANYONE_CAN_PAY_SCRIPT: {
    code_hash: "0xe683b04139344768348499c23eb1326d5a52d6db006c0d2fece00a831f3660d7",
    hash_type: "type",
    args: "0x",
  },
  DEPOSIT_CODE_HASH: <take from capsule deploy logs (data_hash)>,
  SIGHASH_CODE_HASH: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
  ACCOUNT_LOCK_ARGS: <your miner account lock args>,
  BRIDGE_CODE_HASH: <take from capsule deploy logs (data_hash)>,
  RPC: "http://127.0.0.1:8114",
  INDEXER_DATA_PATH: "./indexed-data",
}
```

Now run `npm run run`.
