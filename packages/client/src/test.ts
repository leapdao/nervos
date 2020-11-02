import { BridgeClient, BridgeConfig } from "./client";
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
      tx_hash: "0xa3c3d96042f23411186456f53356dc1c522df92a12814d19c322cce39b5be2fd",
      index: "0x0",
    },
    dep_type: "code",
  },
  DEPOSIT_DEP: {
    out_point: {
      tx_hash: "0x30ce0f1f27112b781a1eb5951d4f5fee4bda0478cfdda767341f9fa6cf56f49d",
      index: "0x0",
    },
    dep_type: "code",
  },
  ANYONE_CAN_PAY_SCRIPT: {
    code_hash: "0xe683b04139344768348499c23eb1326d5a52d6db006c0d2fece00a831f3660d7",
    hash_type: "type",
    args: "0x",
  },
  DEPOSIT_CODE_HASH: "0xd7aa21f5395d0bb03935e88ccd46fd34bd97e1a09944cec3fcb59c340deba6cf",
  SIGHASH_CODE_HASH: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
  ACCOUNT_LOCK_ARGS: "0xa01b3e5d05e2efeb707d3ca0e9fcf9373e87693d",
  BRIDGE_CODE_HASH: "0xe3aed11ce22c8edd787e5aab2601d6f30e3217961d89b0c01b8083b7fcf3e8dd",
  RPC: "http://127.0.0.1:8114",
  INDEXER_DATA_PATH: "./indexed-data",
}

const client = new BridgeClient(myConfig);

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
  await client.deploy(10000n, 1000000000000n, validators, sign);
  await sleep(30000);
  await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 1000000000000n, 10000n, sign);
}

main();
