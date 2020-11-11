import { Indexer, TransactionCollector } from "@ckb-lumos/indexer";
import { Cell, CellDep, Script, HashType, HexString, utils } from "@ckb-lumos/base";
import { RPC } from "ckb-js-toolkit";
import { BridgeConfig } from './client';

type NewDeposit = {
  kind: "new_deposit",
}

type DepositsCollected = {
  kind: "deposits_collected",
}

type BridgeEvent = NewDeposit | DepositsCollected;

type Subscriber = (event: BridgeEvent) => void;

class BridgeEventEmitter {
  private subscribers: Array<Subscriber>;
  private rpc: RPC;
  private indexer: Indexer;
  lastSeenBlock: string;
  readonly bridgeScript: Script;
  readonly config: BridgeConfig;

  constructor(bridgeScript: Script, config: BridgeConfig, indexer: Indexer, rpc: RPC, fromBlock = '0x0') {
    this.config = config;
    this.indexer = indexer;
    this.rpc = rpc;
    this.lastSeenBlock = fromBlock;
    this.bridgeScript = bridgeScript;
    this.subscribers = [];
  }

  //TODO: this assumes no re-orgs i think. How to handle reorgs??
  private async tick() {
    const tip = (await this.indexer.tip()).block_number;

    const bridgeCollector = new TransactionCollector(this.indexer, {
      type: this.bridgeScript,
      fromBlock: this.lastSeenBlock,
      toBlock: tip,
    });
    const depositsCollector = new TransactionCollector(this.indexer, {
      lock: {
        code_hash: this.config.DEPOSIT_CODE_HASH,
        hash_type: "data",
        args: "0x",
      },
      argsLen: "any",
      fromBlock: this.lastSeenBlock,
      toBlock: tip,
    });

    for await (const tx of bridgeCollector.collect()) {

    }

    this.lastSeenBlock = tip;
  }

  subscribe(subscriber: Subscriber) {
    this.subscribers.push(subscriber);
  }

  unsubscribe(subscriber: Subscriber) {
    const index = this.subscribers.indexOf(subscriber);
    this.subscribers.splice(index);
  }
}
