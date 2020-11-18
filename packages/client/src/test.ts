
import { BridgeClient, BridgeConfig } from "./client";
import { Script } from "@ckb-lumos/base";
import { BridgeEventEmitter, BridgeEvent } from "./events";
import { Indexer } from "@ckb-lumos/indexer";
import { RPC } from "ckb-js-toolkit";
import readline from "readline";
import { TransactionSkeletonType } from "@ckb-lumos/helpers";

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
      tx_hash: "0x5632558fda230b41743cf75d11584c88b9ccf6d1386c26ac5a73029dbdb6f8c4",
      index: "0x0",
    },
    dep_type: "code",
  },
  DEPOSIT_DEP: {
    out_point: {
      tx_hash: "0xe9d5c799e5976a604cca8818dfe7686bab6a3e1352a2b06d94ac5917f6cc932d",
      index: "0x0",
    },
    dep_type: "code",
  },
  ANYONE_CAN_PAY_DEP: {
    out_point: {
      tx_hash: "0x44a4bfcc46452889bc524b9dd8057f7232cb82f0d77a088248504299ef97fdb4",
      index: "0x0",
    },
    dep_type: "code",
  },
  ANYONE_CAN_PAY_SCRIPT: {
    code_hash: "0x0e95396c13c9f0dfb48fedfe0dd670eaa228fb8fb6f5a82b8b8dfe89c8c1bb37",
    hash_type: "data",
    args: "0x",
  },
  BRIDGE_SCRIPT: {
    code_hash: '0xc3b8602acaf51a50e6eee26328b73358e4b65e0d56cac0978dc297d8e2a6b4ba',
    hash_type: 'data',
    args: '0x4b45e761b61f887053c417cac7ae7262455385f891007dd08233f4efd7abdc7f00000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee'
  },
  DEPOSIT_CODE_HASH: "0xd7aa21f5395d0bb03935e88ccd46fd34bd97e1a09944cec3fcb59c340deba6cf",
  SIGHASH_CODE_HASH: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
  ACCOUNT_LOCK_ARGS: "0xa01b3e5d05e2efeb707d3ca0e9fcf9373e87693d",
  BRIDGE_CODE_HASH: "0xc3b8602acaf51a50e6eee26328b73358e4b65e0d56cac0978dc297d8e2a6b4ba",
  RPC: "http://127.0.0.1:8114",
  INDEXER_DATA_PATH: "./indexed-data",
}

const rpc = new RPC(myConfig.RPC);
const indexer = new Indexer(myConfig.RPC, myConfig.INDEXER_DATA_PATH);
const emitter = new BridgeEventEmitter(myConfig.BRIDGE_SCRIPT as Script, myConfig, indexer, rpc);
const client = new BridgeClient(myConfig, indexer, rpc, emitter);
emitter.subscribe((e: BridgeEvent) => { console.log("EVENT!!!!!"); console.log(e) });
emitter.start();

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
const sleep = (t: number) => new Promise((resolve) => setTimeout(resolve, t));

async function main() {
  // await client.deploy(10000n, 1000000000000n, validators, sign);
  // console.log(client.BRIDGE_SCRIPT);
  // await sleep(30000);
  console.log(await client.getLatestBridgeState());
  await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 1000000000000n, 10000n, sign);
  // await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 1000000000000n, 10000n, sign);
  // await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 1000000000000n, 10000n, sign);
  const depositsBefore = await client.getDeposits();
  console.log(depositsBefore);
  await client.collectDeposits(depositsBefore, 10000n, myConfig.ACCOUNT_LOCK_ARGS, sign);
  const depositsAfter = await client.getDeposits();
  console.log(depositsAfter);
  console.log(await client.getLatestBridgeState());
}

main();