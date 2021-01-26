
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
      tx_hash: "0x65d29b00078aa7ce884c405f65ee9c6c9694925bc0c1df64b447078888b1922d",
      index: "0x0",
    },
    dep_type: "code",
  },
  DEPOSIT_DEP: {
    out_point: {
      tx_hash: "0xb9cb4d5c92e7ee1b59e4163c539819073ef47dc752ab774817b53a64f6ec126c",
      index: "0x0",
    },
    dep_type: "code",
  },
  AUDIT_DELAY_DEP: {
    out_point: {
      tx_hash: "0xa418913303f3c5b2b4b318b72026af42242578745fa7ec57e1929c99ae5b2884",
      index: "0x0",
    },
    dep_type: "code",
  },
  ANYONE_CAN_PAY_DEP: {
    out_point: {
      tx_hash: "0x3c9264cc331292e664615ea545dd5848ada69a1a82cbf01ef07827f9542cf79e",
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
  BRIDGE_SCRIPT: {
    code_hash: '0xd9d4f57607e5f54ef9c9edcf8c7fdb4304da1d83edfdea258421bc940eb3013f',
    hash_type: 'data',
    args: '0xe658991e1b402bfe94196660f97c8bdf745613b00122c713c34274407cfaf3be00000000ddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee'
  },
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

const validator = ethers.Wallet.createRandom();
const validators: Array<string> = [validator.address];
// const validators: Array<string> = [ethers.constants.HashZero];
const trustee = "0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const sleep = (t: number) => new Promise((resolve) => setTimeout(resolve, t));

async function main() {
  await client.deploy(10000n, 1000000000000n, validators, trustee, signWithPriv);
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

async function makeReceipt(amount: bigint, txHash: string, receiver: string): Promise<Receipt> {
  const raw = ethers.utils.concat([
    ethers.constants.HashZero,
    ethers.utils.hexZeroPad("0x" + amount.toString(16), 32),
    receiver,
    txHash,
  ]);
  const sig = await validator.signMessage(raw);
  const mapV: { [index: string]: string } = {
    "1b": "01",
    "1c": "00",
  }
  return {
    amount: amount,
    txHash: txHash,
    receiver: receiver,
    raw: ethers.utils.hexlify(raw),
    sigs: sig.slice(0, sig.length - 2) + mapV[sig.slice(sig.length - 2)],
  };
}

function replaceChar(origString: string, replaceChar: string, index: number) {
  let firstPart = origString.substr(0, index);
  let lastPart = origString.substr(index + 1);

  let newString = firstPart + replaceChar + lastPart;
  return newString;
}


async function payout() {
  const CK_BYTE = 100000000n;
  const CKB_32 = 32n * CK_BYTE;
  const receiver = "0x0efdcdec4f8490c951e4a225db3bce7274278b5d05be24d7f692454488412ad7";
  const receipt = await makeReceipt(100n * CKB_32, ethers.constants.HashZero, receiver);
  console.log(receipt.raw);


  let privateKey = '0x278a5de700e29faae8e40e366ec5012b5ec63d36ec77e8a2417154cc1d25383f';
  let wallet = new ethers.Wallet(privateKey);
  console.log("KEYS:");
  console.log("address ", wallet.address);
  console.log("pubkey ", wallet._signingKey().publicKey);
  console.log("privkey ", privateKey);

  console.log("SIGS");
  const sig = await wallet.signMessage(ethers.utils.arrayify(receipt.raw));
  console.log(sig);
  console.log(ethers.utils.splitSignature(sig));

  console.log("RECEIPT HASH");
  console.log(hashMessage(ethers.utils.arrayify(receipt.raw)));

  // console.log("RANDOM BYTES: ", ethers.utils.arrayify(privateKey));
  // console.log("RANDOM HASH: ", ethers.utils.keccak256(ethers.utils.arrayify(privateKey)));
  return;
  // return;
  // const receipt1 = {
  //   amount: CKB_32 * 100n,
  //   raw: "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000abca24c5aae65749b577f7405d20f99423295999e2104503614bc098cf3d26186e5785dc9695b7fbfcbfd09843ef338ca8b54c2f8b7da2f4d8a20c23acd7cdc87",
  //   receiver: receiver,
  //   sigs: "0xea25f87dba5122fddffcedbd393b7eac7930d9b05f1ab4f50a93e0615f15e2dd4449c860af32af1b23321593b030bf839b2b8eedfc761b319d0efe15a3193e2800",
  //   txHash: ethers.constants.HashZero,
  // };
  // console.log(receipt1);
  // return;
  console.log(await client.getLatestBridgeState());
  await client.payout(receipt, 10000n, myConfig.ACCOUNT_LOCK_ARGS, signWithPriv);
}

// main();
payout();
