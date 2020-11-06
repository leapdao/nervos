import { Cell, CellDep, Script, HashType, HexString, utils } from "@ckb-lumos/base";
import { Indexer, CellCollector } from "@ckb-lumos/indexer";
import { RPC } from "ckb-js-toolkit";
import { TransactionSkeletonType, TransactionSkeleton, sealTransaction } from "../node_modules/@ckb-lumos/helpers";
import { List } from "immutable";
import { secp256k1Blake160 } from "@ckb-lumos/common-scripts";
import { initializeConfig } from "@ckb-lumos/config-manager";

interface BridgeConfig {
  SIGHASH_DEP: CellDep,
  BRIDGE_DEP: CellDep,
  DEPOSIT_DEP: CellDep,
  ANYONE_CAN_PAY_SCRIPT: Script,
  BRIDGE_SCRIPT?: Script,
  SIGHASH_CODE_HASH: string,
  ACCOUNT_LOCK_ARGS: string,
  BRIDGE_CODE_HASH: string,
  DEPOSIT_CODE_HASH: string,
  RPC: string,
  INDEXER_DATA_PATH: string,
}

const WITNESS_TEMPLATE = "0x55000000100000005500000055000000410000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

class BridgeClient {
  readonly CONFIG: BridgeConfig;
  indexer: Indexer;
  rpc: RPC;
  BRIDGE_SCRIPT?: Script;

  constructor(config: BridgeConfig) {
    this.CONFIG = config;
    this.indexer = new Indexer(this.CONFIG.RPC, this.CONFIG.INDEXER_DATA_PATH);
    this.rpc = new RPC(this.CONFIG.RPC);
    this.BRIDGE_SCRIPT = this.CONFIG.BRIDGE_SCRIPT;

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
    return txHash;
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
