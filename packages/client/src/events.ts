import { Indexer, TransactionCollector } from "@ckb-lumos/indexer";
import { Cell, CellDep, Script, HashType, HexString, utils, Transaction } from "@ckb-lumos/base";
import { RPC } from "ckb-js-toolkit";
import { BridgeConfig } from './client';

type NewDeposit = {
  kind: "new_deposit",
}

type DepositsCollected = {
  kind: "deposits_collected",
}

type BridgeEvent = {
  txHash: string,
  event: NewDeposit | DepositsCollected,
}

type Subscriber = (event: BridgeEvent) => void;

function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

class BridgeEventEmitter {
  private subscribers: Array<Subscriber>;
  private rpc: RPC;
  private indexer: Indexer;
  lastSeenBlock: string;
  readonly bridgeScript: Script;
  readonly config: BridgeConfig;
  halt: boolean;
  readonly refreshInterval: number;

  constructor(bridgeScript: Script, config: BridgeConfig, indexer: Indexer, rpc: RPC, fromBlock = '0x0', refreshInterval = 1000) {
    this.config = config;
    this.indexer = indexer;
    this.rpc = rpc;
    this.lastSeenBlock = fromBlock;
    this.bridgeScript = bridgeScript;
    this.refreshInterval = refreshInterval;
    this.subscribers = [];
    this.halt = true;
  }

  //TODO: this assumes no re-orgs i think. How to handle reorgs??
  private async tick() {
    const tip = (await this.indexer.tip()).block_number;

    if (tip != this.lastSeenBlock) {
      const bridgeCollector = new TransactionCollector(this.indexer, {
        type: this.bridgeScript,
        fromBlock: "0x" + (BigInt(this.lastSeenBlock) + BigInt(1)).toString(16),
        toBlock: tip,
      });
      const depositsCollector = new TransactionCollector(this.indexer, {
        lock: {
          code_hash: this.config.DEPOSIT_CODE_HASH,
          hash_type: "data",
          args: "0x",
        },
        argsLen: "any",
        fromBlock: "0x" + (BigInt(this.lastSeenBlock) + BigInt(1)).toString(16),
        toBlock: tip,
      });

      let depositTxs = [];
      const bridgeTxs = [];
      for await (const tx of depositsCollector.collect()) {
        depositTxs.push(tx);
      }
      for await (const tx of bridgeCollector.collect()) {
        bridgeTxs.push(tx);
      }

      // return allCells.filter(c => {
      //   if (!c.cell_output) return false;
      //   if (!this.CONFIG.BRIDGE_SCRIPT) return false;
      //   const argsTypeHash = "0x" + c.cell_output.lock.args.slice(66);
      //   const bridgeTypeHash = utils.computeScriptHash(this.CONFIG.BRIDGE_SCRIPT);
      //   return argsTypeHash == bridgeTypeHash;
      // });
      // filter deposits not meant for our bridge
      const depositsWithInputs = await Promise.all(depositTxs.map((tx => {
        return (async () => {
          const outpoints = ((tx as any).transaction as Transaction).inputs;
          const inputs = await Promise.all(outpoints.map(outpoint => this.rpc.get_live_cell(outpoint.previous_output, false)));
          return {
            ...tx,
            transaction: {
              ...(tx as any).transaction,
              inputs: inputs,
            },
          }
        })();
      })));
      depositTxs = depositsWithInputs.filter(t => {
        const tx = (t as any).transaction as Transaction;
        const { inputs, outputs } = tx;
        console.log("inputs", inputs);
        console.log("outputs", outputs);
        return true;
      });

      // join and remove duplicates
      const allTxs = Object.values([...depositTxs, ...bridgeTxs].reduce((acc, tx) => {
        return {
          [(tx as any).transaction.hash as string]: tx,
          ...acc,
        }
      }, {}));
      // TODO: sort events properly. Kind of related to the reorg problem.

      // map tx to event

      // call subscriebrs

      this.lastSeenBlock = tip;
    }

    await sleep(this.refreshInterval);
    if (!this.halt) this.tick();
  }

  start() {
    this.halt = false;
    this.tick();
  }

  stop() {
    this.halt = true;
  }

  subscribe(subscriber: Subscriber) {
    this.subscribers.push(subscriber);
  }

  unsubscribe(subscriber: Subscriber) {
    const index = this.subscribers.indexOf(subscriber);
    this.subscribers.splice(index);
  }
}

export { BridgeEventEmitter, BridgeEvent };
