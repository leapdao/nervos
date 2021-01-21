import fs from 'fs';
import { Script } from "@ckb-lumos/base";
import { Indexer } from "@ckb-lumos/indexer";
import { RPC } from "ckb-js-toolkit";

import { LockReceipt } from './utils/types';
import { web3 } from './utils/web3';
import Config from '../RedisConfig';

import { BridgeClient, BridgeConfig } from "../../client/src/client";
import { BridgeEventEmitter, BridgeEvent } from "../../client/src/events";

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
  DEPOSIT_CODE_HASH: "0xd7aa21f5395d0bb03935e88ccd46fd34bd97e1a09944cec3fcb59c340deba6cf",
  SIGHASH_CODE_HASH: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
  ACCOUNT_LOCK_ARGS: "0xdb223ec5ff9194b9e2940ddf1b6f85521b9f9336",
  BRIDGE_CODE_HASH: "0xd9d4f57607e5f54ef9c9edcf8c7fdb4304da1d83edfdea258421bc940eb3013f",
  AUDIT_DELAY_CODE_HASH: "0x0623a96cb7b6ca9dea9ff7ba15fe1fb172852ca11ed8d70124787056aac4d660",
  RPC: "http://127.0.0.1:8114",
  INDEXER_DATA_PATH: "./indexed-data",
}

const bridgeScript = JSON.parse(fs.readFileSync('../../client/bridgeScript.json', 'utf8'))
const rpc = new RPC(myConfig.RPC);
const indexer = new Indexer(myConfig.RPC, myConfig.INDEXER_DATA_PATH);
const emitter = new BridgeEventEmitter(bridgeScript as Script, myConfig, indexer, rpc);
const client = new BridgeClient(myConfig, indexer, rpc, emitter);

class CKBRelay {

  queueRunner: any;
  validatorAddress: string;
  bridgeAddress: string = Config.address;
  bridgeHash: string = Config.bridgeHash;

  constructor(queueRunner: any, validator: string) {
    this.queueRunner = queueRunner;
    this.validatorAddress = validator;
  }

  /**
   * Unlock event transfers funds from bridge to user
   */
  async _processUnLockRelay(message: any) 
  {
    // Structure of message.message should always be json string of LockScript
    const unlockReceipt = await JSON.parse(message.message) as LockReceipt;
    client.payout();
    const result = true; // use helper functions call withdraw
    if (result) {
      await this.queueRunner.deleteMessage({ qname: this.bridgeHash, id: message.id });
    }
  }

  async _relayLock(receipt: LockReceipt) {
    await this.queueRunner.sendMessage({
        qname: this.bridgeAddress, // relay to EVM queue
        message: JSON.stringify(receipt)
    });
  }

  async listen() {    
    emitter.start();
    emitter.subscribe((e: BridgeEvent) => {
      if (e.event.kind === 'new_deposit') {
        console.log("BRIDGE EVENT!!!!!\n\n"); console.log(e);
        this._relayLock({
          isLock: true,
          user: e.event.depositor,
          txHash: e.txHash,
          amount: web3.utils.toHex(e.event.amount.toString())
        });
      }
    });
  }

  /**
   * Process messages from queue
   * At this stage queue will always contain unlock events from evm
   */
  async handle() {
    let message = await this.queueRunner.receiveMessage({ qname: this.bridgeHash });

    while (Object.keys(message).length) {
      await this._processUnLockRelay(message);
      message = await this.queueRunner.receiveMessage({ qname: this.bridgeHash });
    }
  }
}

export default CKBRelay;