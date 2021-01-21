import { web3, TWO_ETH } from './web3';
import Config from '../../RedisConfig';

// TODO:: remove once tests are up won't be needed
const simulate = async () => {
    let accounts = await web3.eth.getAccounts();

    for (let x = 0; x < 3; x++) {
        let tx = await web3.eth.sendTransaction({
            from: accounts[0],
            to: Config.address,
            value: TWO_ETH,
            gasPrice: '20000000000'
          });
        const txHash = tx.transactionHash;
        console.log("We have transaction", txHash);
    }
    process.exit();
}

simulate();