
import { BridgeClient, BridgeConfig } from "./client";
import { Script } from "@ckb-lumos/base";
import { BridgeEventEmitter, BridgeEvent } from "./events";
import { Indexer } from "@ckb-lumos/indexer";
import { RPC } from "ckb-js-toolkit";
import readline from "readline";
import { TransactionSkeletonType } from "@ckb-lumos/helpers";
import { signWithPriv } from "./sign";
import fs from "fs";
// import { Address, AddressType } from '@lay2/pw-core';

const myConfig: BridgeConfig = {
  SIGHASH_DEP: {
    out_point: {
      tx_hash: "0xace5ea83c478bb866edf122ff862085789158f5cbff155b7bb5f13058555b708",
      index: "0x0",
    },
    dep_type: "dep_group",
  },
  BRIDGE_DEP: {
    out_point: {
      tx_hash: "0x28b5aab7c243844d968f69a5ca67701e44b3ce0f6b210e90970facd819a9c4d8",
      index: "0x0",
    },
    dep_type: "code",
  },
  DEPOSIT_DEP: {
    out_point: {
      tx_hash: "0xc6a297f305b12375e38dcd1cf9cfb0de63c1123963ab14cf0a7982a29c7f2f8c",
      index: "0x0",
    },
    dep_type: "code",
  },
  AUDIT_DELAY_DEP: {
    out_point: {
      tx_hash: "0xedb9d970ea568de4bd42e9175e26b144795f24f50ea74809cd77f8cd9c2bb164",
      index: "0x0",
    },
    dep_type: "code",
  },
  ANYONE_CAN_PAY_DEP: {
    out_point: {
      tx_hash: "0x56076842356bde5a466acd50254fbbd05bf9156129755e9916d3211857959bfc",
      index: "0x0",
    },
    dep_type: "code",
  },
  ANYONE_CAN_PAY_SCRIPT: {
    code_hash: "0x0e95396c13c9f0dfb48fedfe0dd670eaa228fb8fb6f5a82b8b8dfe89c8c1bb37",
    hash_type: "data",
    args: "0x",
  },
  // BRIDGE_SCRIPT: {
  //   code_hash: '0xc3b8602acaf51a50e6eee26328b73358e4b65e0d56cac0978dc297d8e2a6b4ba',
  //   hash_type: 'data',
  //   args: '0x4b45e761b61f887053c417cac7ae7262455385f891007dd08233f4efd7abdc7f00000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee'
  // },
  DEPOSIT_CODE_HASH: "0xd7aa21f5395d0bb03935e88ccd46fd34bd97e1a09944cec3fcb59c340deba6cf",
  SIGHASH_CODE_HASH: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
  ACCOUNT_LOCK_ARGS: "0xdb223ec5ff9194b9e2940ddf1b6f85521b9f9336",
  BRIDGE_CODE_HASH: "0xd9d4f57607e5f54ef9c9edcf8c7fdb4304da1d83edfdea258421bc940eb3013f",
  AUDIT_DELAY_CODE_HASH: "0x0623a96cb7b6ca9dea9ff7ba15fe1fb172852ca11ed8d70124787056aac4d660",
  RPC: "http://127.0.0.1:8114",
  INDEXER_DATA_PATH: "./indexed-data",
}

const rpc = new RPC(myConfig.RPC);
const indexer = new Indexer(myConfig.RPC, myConfig.INDEXER_DATA_PATH);
const emitter = new BridgeEventEmitter(myConfig.BRIDGE_SCRIPT as Script, myConfig, indexer, rpc);
const client = new BridgeClient(myConfig, indexer, rpc, emitter);
// emitter.subscribe((e: BridgeEvent) => { console.log("EVENT!!!!!"); console.log(e) });
// emitter.start();

const sign = async (skeleton: TransactionSkeletonType): Promise<Array<string>> => {
  const signOne = (entry: { type: string; index: number; message: string }): Promise<string> => {
    return new Promise((resolve) => {
      const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
      });
      console.log(entry);
      rl.question(`Sign the above please \n`, (sig) => {
        resolve(sig);
      });
    });
  }
  const signingEntries = skeleton.get("signingEntries").toArray();
  const sigs: Array<string> = [];
  for (const entry of signingEntries) {
    const sig = await signOne(entry);
    sigs.push(sig);
  }
  return sigs;
}

const validators: Array<string> = ["0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"];
const trustee = "0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const sleep = (t: number) => new Promise((resolve) => setTimeout(resolve, t));

async function main() {
  if (!fs.existsSync('./bridgeScript.json')) {
    await client.deploy(10000n, 1000000000000n, validators, trustee, signWithPriv);
  } else {
    client.BRIDGE_SCRIPT = JSON.parse(fs.readFileSync('./bridgeScript.json', 'utf8'));
  }
  
  console.log(client.BRIDGE_SCRIPT);
  console.log(await client.getLatestBridgeState());
  await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 1000000000000n, 10000n, signWithPriv);
  await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 2000000000000n, 10000n, signWithPriv);
  await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 3000000000000n, 10000n, signWithPriv);
  const depositsBefore = await client.getDeposits();
  console.log(depositsBefore);
  await client.collectDeposits(depositsBefore, 10000n, myConfig.ACCOUNT_LOCK_ARGS, signWithPriv);
  const depositsAfter = await client.getDeposits();
  console.log(depositsAfter);
  console.log(await client.getLatestBridgeState());
}

main();