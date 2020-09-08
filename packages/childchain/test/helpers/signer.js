/**
 * Copyright (c) 2018-present, Leap DAO (leapdao.org)
 *
 * This source code is licensed under the Mozilla Public License, version 2,
 * found in the LICENSE file in the root directory of this source tree.
 */
 

const ethUtil = require('ethereumjs-util');
const abi = require("ethereumjs-abi");

module.exports = class Signer {
  constructor(isLock, userAddress, amount, txHash) {
    this.isLock = isLock;
    this.userAddress = userAddress;
    this.amount = amount;
    this.txHash = txHash;
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