import * as redis from 'redis';
import { promisify } from 'util';
const RSMQPromise = require('rsmq-promise'); // TODO:: needs wrapper type

import { gasPrice, web3 } from './src/utils/web3';
const Contract = require('web3-eth-contract');
Contract.setProvider('ws://localhost:8546');

import BridgeContract from '../childchain/build/contracts/Bridge.json';
import Config from './config.json';
import EvmRelay from './src/evm';
import CkbRelay from './src/ckb';

const startDB = () => {
    const redisClient = redis.createClient({
      host: Config.redis.host,
      port: Config.redis.port
    });
  
    // NOTE: This is unfortunately how the redis client docs recommend
    // promisifying...
    const db = {
      get: promisify(redisClient.get).bind(redisClient),
      set: promisify(redisClient.set).bind(redisClient),
      quit: promisify(redisClient.quit).bind(redisClient)
    };
  
    console.log("DB has been connected...");
    return db;
}
  
const startQueue = async (qname: string) => {
    const rsmq = new RSMQPromise({
        host: Config.redis.host,
        port: Config.redis.port,
        ns: 'rsmq'
    });
  
    // NOTE: On first run, a queue might not exist yet, so we need to create it.
    try {
        await rsmq.getQueueAttributes({ qname });
      } catch (err) {
        console.log('No matching redis queue found. Creating a new one.');
        try {
          await rsmq.createQueue({ qname });
          console.log('Queue successfully created...');
        } catch (err) {
          console.log(err);
          process.exit(1);
        }
      }
    
    return rsmq;
}

const startService = async () => {
  let accounts = await web3.eth.getAccounts(); // Needs to be from config
  const db = startDB();
  
  const contract = new Contract(BridgeContract.abi, Config.address, {
    from: accounts[0],
    gasPrice
  });
  const evmQueue = await startQueue(Config.address);
  const evmService = new EvmRelay(evmQueue, db, contract, accounts[0]);

  // evmService.handle(); // handle relay from ckb
  // TODO:: move to cron job 30 seconds 
  evmService.listen(); // listen for contract events 

  // TODO:: uncomment once connected to lumos functions
  // const ckbQueue = await startQueue(Config.bridgeHash);
  // const ckbService = new CkbRelay(ckbQueue, accounts[0]);
  // ckbService.listen(); // listen for lock events in bridge contract
  // ckbService.handle(); // handle relay from emv
}

startService();
