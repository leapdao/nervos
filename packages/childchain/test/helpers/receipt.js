/**
 * Copyright (c) 2018-present, Leap DAO (leapdao.org)
 *
 * This source code is licensed under the Mozilla Public License, version 2,
 * found in the LICENSE file in the root directory of this source tree.
 */
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

  getReceipt() {
    return {
      "isLock": this.isLock,
      "user": this.userAddress,
      "amount": this.amount,
      "txHash": this.txHash
    }
  }

  getPayload(validatorAddress) 
  {
    return [this.getReceipt(), this.sign(validatorAddress)];
  }


  // Only use private key during testing. Use public validator address during implementation
  // const sigHash = await this.web3.eth.personal.sign(web3.utils.keccak256(payload), validatorAddress);
  // let sig = sigHash.slice(2);
  // let r = `0x${sig.slice(0, 64)}`;
  // let s = `0x${sig.slice(64, 128)}`;
  // let v = this.web3.utils.hexToNumber(`0x${sig.slice(128, 130)}`);
  sign(validatorAddress) {
    let payload = web3.eth.abi.encodeParameters(
      ["bool", "address", "uint256", "bytes32"],
      [this.isLock, this.userAddress, this.amount, this.txHash]
    );
    
    const sigHash = this.web3.eth.accounts.sign(web3.utils.keccak256(payload), validatorAddress);
    return { v: sigHash.v, r: sigHash.r, s: sigHash.s };
  }
}

module.exports = {
  lockReceipt: Receipt.lockReceipt,
  unlockReceipt: Receipt.unlockReceipt
};