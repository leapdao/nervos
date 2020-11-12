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
      tx_hash: "0x81e5556f3b067a96b42d3b95df310873721b39af223e002fdc46be10d821cc91",
      index: "0x0",
    },
    dep_type: "code",
  },
  DEPOSIT_DEP: {
    out_point: {
      tx_hash: "0xbcd93b454bd9506f0ca041b361e464578372d99f16435518cabf299af461c40f",
      index: "0x0",
    },
    dep_type: "code",
  },
  ANYONE_CAN_PAY_DEP: {
    out_point: {
      tx_hash: "0x0d8814a94a1f13d52351d7f6c01938f34bcbd99f305e8d18a9ae5f724c0b5eb5",
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
  //   args: '0x0baa39a4bc59c288e286050bcc16914edfe8780ff386512f41812ed3cf67350400000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee'
  // },
  DEPOSIT_CODE_HASH: "0xd7aa21f5395d0bb03935e88ccd46fd34bd97e1a09944cec3fcb59c340deba6cf",
  SIGHASH_CODE_HASH: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
  ACCOUNT_LOCK_ARGS: "0xa01b3e5d05e2efeb707d3ca0e9fcf9373e87693d",
  BRIDGE_CODE_HASH: "0xc3b8602acaf51a50e6eee26328b73358e4b65e0d56cac0978dc297d8e2a6b4ba",
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
  console.log(client.BRIDGE_SCRIPT);
  await sleep(30000);
  console.log(await client.getLatestBridgeState());
  await client.deposit(myConfig.ACCOUNT_LOCK_ARGS, 1000000000000n, 10000n, sign);
  await sleep(60000);
  const depositsBefore = await client.getDeposits();
  console.log(depositsBefore);
  await client.collectDeposits(depositsBefore, 10000n, myConfig.ACCOUNT_LOCK_ARGS, sign);
  await sleep(60000);
  const depositsAfter = await client.getDeposits();
  console.log(depositsAfter);
  console.log(await client.getLatestBridgeState());
}

main();
