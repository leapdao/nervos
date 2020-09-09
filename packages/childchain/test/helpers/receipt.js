/**
 * Copyright (c) 2018-present, Leap DAO (leapdao.org)
 *
 * This source code is licensed under the Mozilla Public License, version 2,
 * found in the LICENSE file in the root directory of this source tree.
 */
 

const ethUtil = require('ethereumjs-util');
const abi = require("ethereumjs-abi");

const Receipt = class Receipt {
  constructor(isLock, userAddress, amount, txHash) {
    this.isLock = isLock;
    this.userAddress = userAddress;
    this.amount = amount;
    this.txHash = txHash;
    this.web3 = web3;
  }

  static lockReceipt(address, amount, hash, web3) {
    // todo: check inputs
    return new Receipt(true, address, amount, hash, web3);
  }

  static unlockReceipt(address, amount, hash, web3) {
    return new Receipt(false, address, amount, hash, web3);
  }

  getAbiReceipt() {
    return this.web3.eth.abi.encodeParameter(
      {
        "Receipt": {
          "isLock": 'bool',
          "user": 'address',
          "amount": 'uint256',
          "txHash": 'bytes32'
        }
      }, {
        "isLock": this.isLock,
        "user": this.userAddress,
        "amount": this.amount,
        "txHash": this.txHash
      }
    );
  }

  getAbiSig(privKey) {
    const sig = this.sign(privKey);
    return this.web3.eth.abi.encodeParameter(
      {
        "Sig": {
          "v": 'uint8',
          "r": 'bytes32',
          "s": 'bytes32'
        }
      }, {
        "v": sig.v,
        "r": sig.r,
        "s": sig.s
      }
    );
  }

  sign(privKey) {
    let payload = abi.rawEncode(
      ["bool", "address", "uint256", "bytes32"],
      [this.isLock, this.userAddress, this.amount, this.txHash]
    );
    const sigHash = ethUtil.keccak256(payload);
    const sig = ethUtil.ecsign(
      sigHash,
      Buffer.from(privKey.replace("0x", ""), "hex")
    );
    return sig;
  }
}

module.exports = {
  lockReceipt: Receipt.lockReceipt,
  unlockReceipt: Receipt.unlockReceipt
};