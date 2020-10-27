import { web3 } from './utils/web3';
import { LockReceipt } from './utils/types';
const Contract = require('web3-eth-contract');
Contract.setProvider('ws://localhost:8546');
import Config from '../config.json';

class EVMRelay {
    db: any;
    queueRunner: any;
    contractInstance: any;
    validatorAddress: string;
    bridgeAddress: string = Config.address;
    bridgeHash: string = Config.bridgeHash;

    constructor(queueRunner: any, db: any, contract: any, validator: string) {
        this.db = db;
        this.queueRunner = queueRunner;
        this.validatorAddress = validator;
        this.contractInstance = contract;
    }

    async _getDBHeight(): Promise<number> {
        const result = await this.db.get('evm_height');
        return result !== null ? parseInt(result, 10) : 0;
    }

    async _getEvmHeight(): Promise<number> {
        return (await web3.eth.getBlock("latest")).number;
    }

    async _getSignature(receipt: LockReceipt) {
        let payload = web3.eth.abi.encodeParameters(
            ["bool", "address", "uint256", "bytes32"],
            [receipt.isLock, receipt.user, receipt.amount, receipt.txHash]
        );

        let sig = await web3.eth.sign(web3.utils.keccak256(payload), this.validatorAddress);

        var r = `0x${sig.slice(2, 64 + 2)}`;
        var s = `0x${sig.slice(64 + 2, 128 + 2)}`;
        var v = `0x${sig.slice(128 + 2, 130 + 2)}`;

        return { v, r, s };
    }

    async _collectLock(receipt: LockReceipt) {
        const signature = await this._getSignature(receipt);
        return await this.contractInstance.methods
            .collect(receipt, signature)
            .call();
    }

    // TODO:: check on failure of collecting signatures
    async _collectUnLock(receipt: LockReceipt) {
        const signature = await this._getSignature(receipt);
        this.contractInstance.methods
            .collect(receipt, signature)
            .call();
    }

    async _processEvents(localHeight: number, remoteHeight: number) {
        await this.contractInstance.getPastEvents({
            fromBlock: localHeight,
            toBlock: remoteHeight
        }, (error: any, event: any) => {
                event.forEach((x: any) => {
                if (x.event === 'Burn') {
                    this._collectUnLock({
                        isLock: false,
                        user: x.returnValues.sender,
                        amount: x.returnValues.value,
                        txHash: x.transactionHash
                    });
                } else if (x.event === 'BurnQuorom') {
                    this._relayUnLock({
                        user: x.returnValues.from,
                        amount: x.returnValues.amount,
                        txHash: x.returnValues.txHash,
                        sigs: x.returnValues.from
                    });
                }
            });
        });
    }

    async _processLockRelay(message: any) {
        // Structure of message.message should always be json string of LockScript
        const result = await this._collectLock(JSON.parse(message.message));
        if (result) { // TODO:: check if successfull
          await this.queueRunner.deleteMessage({ qname: this.bridgeAddress, id: message.id });
        }
    }

    async _relayUnLock(evmEvent: any) {
        return this.queueRunner.sendMessage({
            qname: this.bridgeHash, // Should be ckb queue name bridge hash
            message: JSON.stringify(evmEvent)
        });
    }
    
    async listen() {
        console.log("Listening on childchain...");
        const localHeight = await this._getDBHeight();
        const remoteHeight = await this._getEvmHeight();

        console.log("Blocks to process are", (remoteHeight - localHeight));
        if ((remoteHeight - localHeight) > 0) {
            await this._processEvents(localHeight, remoteHeight);
            await this.db.set('evm_height', remoteHeight);
        }

        setTimeout(async () => {
            await this.listen();
        }, 30000)// listen again after 30 seconds
    }

    /**
     * Process messages from queue
     * At this stage queue will always contain lock events from ckb
     */
    async handle() {
        let message = await this.queueRunner.receiveMessage({ qname: this.bridgeAddress });

        while (Object.keys(message).length) {
            await this._processLockRelay(message);
            message = await this.queueRunner.receiveMessage({ qname: this.bridgeAddress });
        }
    }
}

export default EVMRelay;