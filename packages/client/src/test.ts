
import { BridgeClient, BridgeConfig, Receipt } from "./client";
import { Script } from "@ckb-lumos/base";
import { BridgeEventEmitter, BridgeEvent } from "./events";
import { Indexer } from "@ckb-lumos/indexer";
import { RPC } from "ckb-js-toolkit";
import readline from "readline";
import { TransactionSkeletonType } from "@ckb-lumos/helpers";
import { signWithPriv } from "./sign";
import { ethers } from 'ethers';
import { hashMessage } from "@ethersproject/hash";

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
      tx_hash: "0xd1c4b1e44b4d69d2423cc4fad8f23bb043d8abf9499b1f011fc6463b171a4ad9",
      index: "0x0",
    },
    dep_type: "code",
  },
  DEPOSIT_DEP: {
    out_point: {
      tx_hash: "0x67a4c3a504f73cfc317b0099b3360b209239a91effe9a5ba5c7b933d5ce8087a",
      index: "0x0",
    },
    dep_type: "code",
  },
  AUDIT_DELAY_DEP: {
    out_point: {
      tx_hash: "0xfe32592dd738eb53a92ba1baaa8b4f904bc7cb22e22dce7486ee73601fd48499",
      index: "0x0",
    },
    dep_type: "code",
  },
  ANYONE_CAN_PAY_DEP: {
    out_point: {
      tx_hash: "0xfdfff8120aefc180dcea4ce0fa9a0912578a1bb043384b980c80913596a39672",
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
  ACCOUNT_LOCK_ARGS: "0xa01b3e5d05e2efeb707d3ca0e9fcf9373e87693d",
  BRIDGE_CODE_HASH: "0xd9d4f57607e5f54ef9c9edcf8c7fdb4304da1d83edfdea258421bc940eb3013f",
  AUDIT_DELAY_CODE_HASH: "0x0623a96cb7b6ca9dea9ff7ba15fe1fb172852ca11ed8d70124787056aac4d660",
  RPC: "http://127.0.0.1:8114",
  INDEXER_DATA_PATH: "./indexed-data",
  TIMEOUT: 100n,
}

const rpc = new RPC(myConfig.RPC);
const indexer = new Indexer(myConfig.RPC, myConfig.INDEXER_DATA_PATH);
const emitter = new BridgeEventEmitter(myConfig.BRIDGE_SCRIPT as Script, myConfig, indexer, rpc);
const client = new BridgeClient(myConfig, indexer, rpc, emitter);
emitter.subscribe((e: BridgeEvent) => { console.log("EVENT!!!!!"); console.log(e) });
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

async function makeReceipt(amount: bigint, txHash: string, receiver: string): Promise<Receipt> {
  const raw = ethers.utils.concat([
    ethers.constants.HashZero,
    ethers.utils.hexZeroPad("0x" + amount.toString(16), 32),
    receiver,
    txHash,
  ]);
  const sig = await validator.signMessage(raw);
  const mapV: { [index: string]: string } = {
    "1b": "00",
    "1c": "01",
  }
  return {
    amount: amount,
    txHash: txHash,
    receiver: receiver,
    raw: ethers.utils.hexlify(raw),
    sigs: sig.slice(0, sig.length - 2) + mapV[sig.slice(sig.length - 2)],
  };
}

let privateKey = '0x278a5de700e29faae8e40e366ec5012b5ec63d36ec77e8a2417154cc1d25383f';
let validator = new ethers.Wallet(privateKey);
const validators: Array<string> = [validator.address];
const trustee = "0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const sleep = (t: number) => new Promise((resolve) => setTimeout(resolve, t));

async function main() {
  await client.deploy(10000n, 1000000000000n, validators, trustee, signWithPriv);
  console.log(client.BRIDGE_SCRIPT);
  console.log(await client.getLatestBridgeState());
  // await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 1000000000000n, 10000n, signWithPriv);
  // await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 2000000000000n, 10000n, signWithPriv);
  await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 3000000000000n, 10000n, signWithPriv);
  const depositsBefore = await client.getDeposits();
  console.log(depositsBefore);
  await client.collectDeposits(depositsBefore, 10000n, myConfig.ACCOUNT_LOCK_ARGS, signWithPriv);
  const depositsAfter = await client.getDeposits();
  console.log(depositsAfter);
  console.log(await client.getLatestBridgeState());
  const CK_BYTE = 100000000n;
  const CKB_32 = 32n * CK_BYTE;
  const receiver = "0x0efdcdec4f8490c951e4a225db3bce7274278b5d05be24d7f692454488412ad7";
  const receipt = await makeReceipt(100n * CKB_32, ethers.constants.HashZero, receiver);
  await client.payout(receipt, 10000n, myConfig.ACCOUNT_LOCK_ARGS, signWithPriv);

  console.log(await client.getLatestBridgeState());
}

main();
