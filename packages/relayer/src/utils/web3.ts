import Web3 from 'web3';

// TODO:: read websocket url from .env
export const web3 = new Web3('http://localhost:8545');
export const gasPrice = '20000000000';
const { BN } = require("ethereumjs-util");
export const TWO_ETH = new BN('200000000000000000', 10);
