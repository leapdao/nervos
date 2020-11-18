import { Cell, CellDep, Script, HashType, HexString, utils } from "@ckb-lumos/base";
import { Indexer, CellCollector } from "@ckb-lumos/indexer";
import { RPC } from "ckb-js-toolkit";
import { TransactionSkeletonType, TransactionSkeleton, sealTransaction, createTransactionFromSkeleton } from "@ckb-lumos/helpers";
import { List } from "immutable";
import { secp256k1Blake160 } from "@ckb-lumos/common-scripts";
import { initializeConfig } from "@ckb-lumos/config-manager";
import { BridgeEventEmitter, Subscriber, BridgeEvent } from "./events";

interface BridgeConfig {
  SIGHASH_DEP: CellDep,
  BRIDGE_DEP: CellDep,
  DEPOSIT_DEP: CellDep,
  ANYONE_CAN_PAY_DEP: CellDep,
  ANYONE_CAN_PAY_SCRIPT: Script,
  BRIDGE_SCRIPT?: Script,
  SIGHASH_CODE_HASH: string,
  BRIDGE_CODE_HASH: string,
  DEPOSIT_CODE_HASH: string,
  ACCOUNT_LOCK_ARGS: string,
  RPC: string,
  INDEXER_DATA_PATH: string,
}

interface BridgeState {
  stateid: string,
  validators: Array<string>,
  capacity: bigint,
}

const WITNESS_TEMPLATE = "0x55000000100000005500000055000000410000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

class BridgeClient {
  readonly CONFIG: BridgeConfig;
  indexer: Indexer;
  rpc: RPC;
  eventEmitter: BridgeEventEmitter;
  BRIDGE_SCRIPT?: Script;

  constructor(config: BridgeConfig, indexer: Indexer, rpc: RPC, eventEmitter: BridgeEventEmitter) {
    this.CONFIG = config;
    this.indexer = indexer;
    this.rpc = rpc;
    this.BRIDGE_SCRIPT = this.CONFIG.BRIDGE_SCRIPT;
    this.eventEmitter = eventEmitter;

    initializeConfig();
    this.indexer.startForever();
  }

  // add event emitter
  // add close call on client
  // actually await txes

  async deploy(fee: bigint, initialCapacity: bigint, validators: Array<string>, sign: (skeleton: TransactionSkeletonType) => Promise<Array<string>>): Promise<string> {
    const collector = new CellCollector(this.indexer, {
      lock: {
        code_hash: this.CONFIG.SIGHASH_CODE_HASH,
        hash_type: "type",
        args: this.CONFIG.ACCOUNT_LOCK_ARGS,
      },
    });

    const allCells = [];
    for await (const cell of collector.collect()) {
      allCells.push(cell);
    }

    const fundingCell = allCells.find(cell => BigInt(cell.cell_output.capacity) >= initialCapacity + fee);
    if (!fundingCell) {
      throw new Error("No cell with enough capacity belongs to account!");
    }
    if (!fundingCell.out_point) {
      throw new Error("No outpoint on funding cell!");
    }
    const inputs = [fundingCell];

    const stateId = fundingCell.out_point.tx_hash + fundingCell.out_point.index.replace("0x", "").padStart(8, "0");
    const bridgeScript = {
      code_hash: this.CONFIG.BRIDGE_CODE_HASH,
      hash_type: "data" as HashType,
      args: stateId + validators.map(v => v.replace("0x", "")).reduce((acc, cv) => cv + acc, ""),
    }
    const bridgeOutput = {
      cell_output: {
        capacity: "0x" + initialCapacity.toString(16),
        lock: this.CONFIG.ANYONE_CAN_PAY_SCRIPT,
        type: bridgeScript,
      },
      data: "0x",
    }
    const outputs =
      BigInt(fundingCell.cell_output.capacity) == initialCapacity + fee ?
        [bridgeOutput] :
        [
          bridgeOutput,
          {
            cell_output: {
              capacity: "0x" + (BigInt(fundingCell.cell_output.capacity) - initialCapacity - fee).toString(16),
              lock: fundingCell.cell_output.lock,
            },
            data: "0x",
          }
        ];

    const deps = [this.CONFIG.SIGHASH_DEP, this.CONFIG.BRIDGE_DEP];
    const witnesses = [WITNESS_TEMPLATE];

    let skeleton = this.makeSkeleton(inputs, outputs, deps, witnesses);
    skeleton = secp256k1Blake160.prepareSigningEntries(skeleton);
    const sigs = await sign(skeleton);
    const tx = sealTransaction(skeleton, sigs);
    const txHash = await this.rpc.send_transaction(tx);

    // this feels very hackish
    this.eventEmitter.bridgeScript = bridgeScript;
    this.eventEmitter.start();

    const subscriber = await this.awaitTransaction(txHash);
    this.eventEmitter.unsubscribe(subscriber);

    this.BRIDGE_SCRIPT = bridgeScript;
    return txHash;
  }

  async deposit(lockArgs: string, amount: bigint, fee: bigint, sign: (skeleton: TransactionSkeletonType) => Promise<Array<string>>): Promise<string> {
    if (!this.BRIDGE_SCRIPT) {
      throw new Error("No bridge set on client!");
    }

    const [inputs, sum] = await this.collectEnoughCells(lockArgs, amount);

    const lockHash = utils.computeScriptHash({
      code_hash: this.CONFIG.SIGHASH_CODE_HASH,
      hash_type: "type",
      args: lockArgs,
    });
    const depositOutput = {
      cell_output: {
        capacity: "0x" + amount.toString(16),
        lock: {
          code_hash: this.CONFIG.DEPOSIT_CODE_HASH,
          hash_type: "data" as HashType,
          args: lockHash + utils.computeScriptHash(this.BRIDGE_SCRIPT).replace("0x", ""),
        },
      },
      data: "0x",
    }
    const outputs =
      sum == amount + fee ?
        [depositOutput] :
        [
          depositOutput,
          {
            cell_output: {
              capacity: "0x" + (sum - amount - fee).toString(16),
              lock: inputs[0].cell_output.lock,
            },
            data: "0x",
          }
        ];

    const deps = [this.CONFIG.SIGHASH_DEP];
    const witnesses = Array(inputs.length).fill(WITNESS_TEMPLATE);

    let skeleton = this.makeSkeleton(inputs, outputs, deps, witnesses);
    skeleton = secp256k1Blake160.prepareSigningEntries(skeleton);
    const sigs = await sign(skeleton);
    const tx = sealTransaction(skeleton, sigs);
    const txHash = await this.rpc.send_transaction(tx);
    const subscriber = await this.awaitTransaction(txHash);
    this.eventEmitter.unsubscribe(subscriber);
    return txHash;
  }

  async collectDeposits(deposits: Array<Cell>, fee: bigint, funderLockArgs: string, sign: (skeleton: TransactionSkeletonType) => Promise<Array<string>>): Promise<string> {
    const bridgeCell = await this.getLatestBridge();
    const [feeCells, feeAmount] = await this.collectEnoughCells(funderLockArgs, fee);
    const inputs = [bridgeCell, ...deposits, ...feeCells];

    const depositAmount = deposits.map(c => BigInt(c.cell_output.capacity)).reduce((acc, cv) => acc + cv, 0n);

    const outputBridgeCell = {
      data: bridgeCell.data,
      cell_output: {
        ...bridgeCell.cell_output,
        capacity: "0x" + (BigInt(bridgeCell.cell_output.capacity) + depositAmount).toString(16),
      },
    };
    const outputs =
      feeAmount == fee ?
        [outputBridgeCell] :
        [
          outputBridgeCell,
          {
            cell_output: {
              capacity: "0x" + (feeAmount - fee).toString(16),
              lock: feeCells[0].cell_output.lock,
            },
            data: "0x",
          }
        ];

    const deps = [this.CONFIG.DEPOSIT_DEP, this.CONFIG.SIGHASH_DEP, this.CONFIG.BRIDGE_DEP, this.CONFIG.ANYONE_CAN_PAY_DEP];
    const witnesses = ["0x01", ...Array(deposits.length).fill("0x"), ...Array(feeCells.length).fill(WITNESS_TEMPLATE)];

    let skeleton = this.makeSkeleton(inputs, outputs, deps, witnesses);
    skeleton = secp256k1Blake160.prepareSigningEntries(skeleton);
    const sigs = await sign(skeleton);
    const tx = sealTransaction(skeleton, sigs);
    const txHash = await this.rpc.send_transaction(tx);
    const subscriber = await this.awaitTransaction(txHash);
    this.eventEmitter.unsubscribe(subscriber);
    return txHash;
  }

  async getDeposits(): Promise<Array<Cell>> {
    const collector = new CellCollector(this.indexer, {
      lock: {
        code_hash: this.CONFIG.DEPOSIT_CODE_HASH,
        hash_type: "data",
        args: "0x",
      },
      argsLen: "any",
    });

    const allCells = [];
    for await (const cell of collector.collect()) {
      allCells.push(cell);
    }
    return allCells.filter(c => {
      if (!c.cell_output) return false;
      if (!this.BRIDGE_SCRIPT) return false;
      const argsTypeHash = "0x" + c.cell_output.lock.args.slice(66);
      const bridgeTypeHash = utils.computeScriptHash(this.BRIDGE_SCRIPT);
      return argsTypeHash == bridgeTypeHash;
    });
  }

  async getLatestBridge(): Promise<Cell> {
    if (!this.BRIDGE_SCRIPT) {
      throw new Error("Bridge script not set!");
    }

    const collector = new CellCollector(this.indexer, {
      type: this.BRIDGE_SCRIPT,
    });

    const allCells = [];
    for await (const cell of collector.collect()) {
      allCells.push(cell);
    }

    return allCells[0];
  }

  async getLatestBridgeState(): Promise<BridgeState> {
    const latestCell = await this.getLatestBridge();
    if (!latestCell.cell_output) {
      throw new Error("No cell_output on bridge cell");
    }
    if (!latestCell.cell_output.type) {
      throw new Error("No type script on bridge cell");
    }
    const typeArgs = latestCell.cell_output.type.args;
    const stateid = typeArgs.slice(0, 74);
    const validatorsString = typeArgs.slice(74);
    const validators = [];
    for (let i = 0; i < validatorsString.length / 40; i++) {
      validators.push("0x" + validatorsString.slice(i * 40, i + 40));
    }
    return {
      stateid: stateid,
      validators: validators,
      capacity: BigInt(latestCell.cell_output.capacity),
    };
  }

  awaitTransaction(txHash: string): Promise<Subscriber> {
    return new Promise((resolve) => {
      function subscriber(event: BridgeEvent): void {
        if (event.txHash === txHash) resolve(subscriber);
      }
      this.eventEmitter.subscribe(subscriber);
    });
  }

  private makeSkeleton(inputs: Array<Cell>, outputs: Array<Cell>, deps: Array<CellDep>, witnesses: Array<HexString>): TransactionSkeletonType {
    return TransactionSkeleton({
      cellProvider: this.indexer,
      inputs: List(inputs),
      outputs: List(outputs),
      cellDeps: List(deps),
      witnesses: List(witnesses),
    });
  }

  private async collectEnoughCells(lockArgs: string, amount: bigint): Promise<[Array<Cell>, bigint]> {
    const collector = new CellCollector(this.indexer, {
      lock: {
        code_hash: this.CONFIG.SIGHASH_CODE_HASH,
        hash_type: "type",
        args: lockArgs,
      },
    });

    const allCells = [];
    for await (const cell of collector.collect()) {
      allCells.push(cell);
    }

    let { sum, cells } = allCells.reduce(({ sum, cells }, cell) => {
      if (sum >= amount) return { sum, cells };
      return {
        cells: [cell, ...cells],
        sum: sum + BigInt(cell.cell_output.capacity),
      };
    }, { sum: 0n, cells: [] as Array<Cell> });
    return [cells, sum];
  }
}

export { BridgeClient, BridgeConfig };
