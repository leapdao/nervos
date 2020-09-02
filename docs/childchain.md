# Childchain Spec

The childchain is an Ethermint chain running as [a sidechain](https://talk.nervos.org/t/ckb-sidechain-framework/4722) to the CKB mainnet.

the bridged token is used as native token of the sidechain.
the bridge contract needs to be deployed with some tokens, which then are distributed among the validators. 

## Setup

- generating keyfiles
- aggregating genesis file
- launching network
- deploying contracts

## Gathering lock signatures and minting tokens

```
// function checks at any submission if quorum reached, then calls
// executeMint(txHash);
collectLock(address to, uint256 amount, bytes32 txHash, bytes memory sig);
```

## Burning tokens and gathering unlock signatures

// receives native tokens
burn() payable;

collectUnlock(address from, uint256 amount, bytes32 txHash, bytes memory sig);
-> Event is emitted when quorum of sigs collected, which then needs to be relayed to CKB bridge.