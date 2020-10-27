import { web3 } from './src/utils/web3';
import * as fs from 'fs';
// TODO:: handle if not available
import BridgeContract from '../childchain/build/contracts/Bridge.json';
const Contract = require('web3-eth-contract');

// set provider for all later instances to use
Contract.setProvider('ws://localhost:8546')

const deployContract = async () => {
    let accounts = await web3.eth.getAccounts();
    let contractAddress = '';

    const contract = new Contract(BridgeContract.abi);
    contract.deploy({
        data: BridgeContract.bytecode,
        arguments: [accounts]
    })
    .send({
        from: accounts[0],
        gas: 1500000,
        gasPrice: '30000000000000'
    })
    .then(function (newContractInstance: any) {
        contractAddress = newContractInstance.options.address;
        const jsonString = JSON.stringify({ address: contractAddress });
        fs.writeFileSync('./address.json', jsonString);
        process.exit();
    });
}

deployContract();

// TODO:: can be the entry point to build all config values
// e.g fetching bridge hash. deploying new contract if new childchain etc