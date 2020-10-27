import { LockReceipt } from './utils/types';
import { web3 } from './utils/web3';
import Config from '../config.json';

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
    // Structure of message.message will be json string of UnlockReceipt

    const result = true; // use helper functions call withdraw
    if (result) {
      await this.queueRunner.deleteMessage({ qname: this.bridgeHash, id: message.id });
    }
  }

  async _relayLock(receipt: LockReceipt) {
    return await this.queueRunner.sendMessage({
        qname: this.bridgeAddress, // relay to EVM queue
        message: JSON.stringify(receipt)
    });
  }

  async listen() {
    // Filter through indexer for deposits on the bridge
    // build receipt object and relay 
    // TODO:: replace test data below with actual deposits
    this._relayLock({
      isLock: true,
      user: this.validatorAddress,
      txHash: "testTxHash",
      amount: web3.utils.toHex('200000000000000000')
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