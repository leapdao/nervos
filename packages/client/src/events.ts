import { Indexer, TransactionCollector } from "@ckb-lumos/indexer";
import { Output, OutPoint, Script, Transaction, Cell, utils } from "@ckb-lumos/base";
import { RPC } from "ckb-js-toolkit";
import { BridgeConfig } from './client';

type NewDeposit = {
  kind: "new_deposit",
  amount: BigInt,
  depositor: string,
}

type DepositsCollected = {
  kind: "deposits_collected",
  deposits: Array<{
    depositor: string,
    amount: BigInt,
  }>,
}

type BridgeDeployed = {
  kind: "bridge_deployed",
  initialCapacity: BigInt,
}

type BridgeEvent = {
  txHash: string,
  event: NewDeposit | DepositsCollected | BridgeDeployed,
}

type Subscriber = (event: BridgeEvent) => void;

type DetailedTx = {
  blockNumber: number,
  transaction: Transaction,
  tx_status: {
    block_hash: string,
    status: "pending" | "proposed" | "commited",
  },
}

function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

class BridgeEventEmitter {
  // make this private maybe
  subscribers: Array<Subscriber>;
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
      let bridgeTxs = [];
      for await (const tx of depositsCollector.collect()) {
        depositTxs.push(tx);
      }
      for await (const tx of bridgeCollector.collect()) {
        bridgeTxs.push(tx);
      }
      // filter deposits not meant for our bridge
      const outpointToCell = async (outpoint: OutPoint): Promise<Cell> => {
        const tx = (await this.rpc.get_transaction(outpoint.tx_hash)).transaction
        return tx.outputs[parseInt(outpoint.index, 16)];
      };
      const dereferenceOutpoints = async (txs: Array<{ transaction: Transaction }>) => {
        return await Promise.all(txs.map((tx => {
          return (async () => {
            const outpoints = (tx.transaction as Transaction).inputs;
            const inputs = await Promise.all(outpoints.map(o => outpointToCell(o.previous_output)));
            return {
              ...tx,
              transaction: {
                ...(tx as any).transaction,
                inputs: inputs,
              },
            }
          })();
        })));
      }
      const depositsWithInputs = await dereferenceOutpoints(depositTxs as any);
      depositTxs = depositsWithInputs.filter((t: any) => {
        const { inputs, outputs } = t.transaction;
        const cellIsADepositToOurBridge = (c: any) => {
          if (!(c.lock.code_hash == this.config.DEPOSIT_CODE_HASH)) return false;
          const argsTypeHash = "0x" + c.lock.args.slice(66);
          const bridgeTypeHash = utils.computeScriptHash(this.bridgeScript);
          return argsTypeHash == bridgeTypeHash;
        };
        return inputs.some(cellIsADepositToOurBridge) || outputs.some(cellIsADepositToOurBridge);
      });
      bridgeTxs = await dereferenceOutpoints(bridgeTxs as any);

      // join and remove duplicates
      const allTxs = Object.values([...depositTxs, ...bridgeTxs].reduce((acc, tx) => {
        return {
          [(tx as any).transaction.hash as string]: tx,
          ...acc,
        }
      }, {}));
      const allTxsWithBlockNum = await Promise.all(allTxs.map((tx: any) => {
        return (async () => {
          const blockHeader = await this.rpc.get_header(tx.tx_status.block_hash);
          return {
            blockNumber: parseInt(blockHeader.number, 16),
            ...tx
          }
        })();
      }));
      // TODO, use tx-index also
      allTxsWithBlockNum.sort((tx1, tx2) => tx1.blockNumber - tx2.blockNumber);
      // TODO: sort events properly. Kind of related to the reorg problem.

      // map tx to event and call subscirbers
      const events = allTxsWithBlockNum.map(this.txToEvent, this);
      for (let event of events) {
        for (let subscriber of this.subscribers) {
          subscriber(event);
        }
      }

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

  private txToEvent(tx: DetailedTx): BridgeEvent {
    // if it has no bridge in inputs, but a bridge in outputs, it is a deploy
    // if it has a deposit lock in the outputs, it is a deposit
    // if there is bridge and deposit in the inputs, it is a collect deposit
    const bridgeInInputs = tx.transaction.inputs.some(input => {
      // TODO: fix this retardation
      let inputCell = input as any;
      if (!inputCell.type) return false;
      return utils.computeScriptHash(this.bridgeScript) === utils.computeScriptHash(inputCell.type);
    });
    const bridgeInOutputs = tx.transaction.outputs.some(output => {
      // TODO: fix this retardation
      let outputCell = output as any;
      if (!outputCell.type) return false;
      return utils.computeScriptHash(this.bridgeScript) === utils.computeScriptHash(outputCell.type);
    });

    // this means bridge deployment
    if (!bridgeInInputs && bridgeInOutputs) {
      const initialBridgeCell = tx.transaction.outputs.find(output => {
        if (!output.type) return false;
        return utils.computeScriptHash(this.bridgeScript) == utils.computeScriptHash(output.type)
      });
      if (!initialBridgeCell) throw new Error("This state can not happen, but TS typeschecker is not smart enough to figure it out");
      return {
        txHash: tx.transaction.hash as string,
        event: {
          kind: "bridge_deployed",
          initialCapacity: BigInt(initialBridgeCell.capacity),
        }
      };
    }

    const cellIsADepositToOurBridge = (c: any) => {
      if (!(c.lock.code_hash == this.config.DEPOSIT_CODE_HASH)) return false;
      const argsTypeHash = "0x" + c.lock.args.slice(66);
      const bridgeTypeHash = utils.computeScriptHash(this.bridgeScript);
      return argsTypeHash == bridgeTypeHash;
    };
    const depositInOutputs = tx.transaction.outputs.find(cellIsADepositToOurBridge);
    // this means a deposit to our bridge
    if (depositInOutputs) {
      const depositAmount = BigInt(depositInOutputs.capacity);
      const depositor = depositInOutputs.lock.args.slice(0, 66);
      return {
        txHash: tx.transaction.hash as string,
        event: {
          kind: "new_deposit",
          amount: depositAmount,
          depositor: depositor,
        },
      };
      // else it is collect deposits (for now)
    } else {
      const depositsInInputs = tx.transaction.inputs.filter(cellIsADepositToOurBridge);
      return {
        txHash: tx.transaction.hash as string,
        event: {
          kind: "deposits_collected",
          deposits: depositsInInputs.map(output => {
            return {
              depositor: (output as any).lock.args.slice(0, 66),
              amount: BigInt((output as any).capacity),
            }
          }),
        },
      };
    }
  }
}

export { BridgeEventEmitter, BridgeEvent, Subscriber };
