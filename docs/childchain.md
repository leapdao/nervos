# Childchain Spec

The childchain is an Ethermint chain running as [a sidechain](https://talk.nervos.org/t/ckb-sidechain-framework/4722) to the CKB mainnet.

The token locked into the bridge on the parent chain is used as native token of the sidechain.

![child chain relationship](https://talk.nervos.org/uploads/default/original/2X/6/6afa0e30c538d6b62aadeb8ce8371dbee963517a.png)

The childchain will hold the full supply of the token initially in the bridge contract. An initial gas supply for the validators will need to be awarded through staking.

## Message Format

The messages are each composed of the following parameters:

**isLockReceipt:** A boolean value determining the direction of transfer. `true` if a lock event is relayed from parent chain to child chain. `false` if a burn event is relayed from the child chain to the parent chain.

**txHash:** 32 bytes hash of the transaction causing the lock or burn event.

**receiver:** an array of 20 bytes holding the address of the sender of the lock transaction or the sender of the burn transactions. The funds will be issued to this address on the other side of the bridge.

**amount:** an unsigned 32 bytes big-endian value encoding the amount of tokens being locked or having been burned. as CKBytes are encoded with 8 decimals, while the child chain native token is encoded with 18 decimals, a conversion needs to be done. when converting from child-chain to parent chain representation, the amount is always rounded down.

The message is constructed by cancatanation of parameters into a byte array. The solidity equivalent 

`abi.encode(bool isLockMsg, bytes32 txHash, address receiver, uint256 amount)`

The resulting byte-array is the passed to `keccak256()` for hashing.

### Lock


an example payload is:

```
isLockReceipt: true,
txHash: 0x1122334411223344112233441122334411223344112233441122334411223344,
receiver: 0x1122334455667788990011223344556677889900,
amount: 1234
```

The payload yields the hash: `0x5e574a0db0614e5279360da4b975c3aa476f11b49ea6fe91ddfdfbf8cc8783a3`

The byte-array encoding of the payload passed to keccak256 is:
`0x00000000000000000000000000000000000000000000000000000000000000011122334411223344112233441122334411223344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2`


### Unlock

an example payload is:

```
isLockReceipt: false,
txHash: 0x3344112233441122334411223344112233441122334411223344112233441122,
receiver: 0x6677889900112233445566778899001122334455,
amount: 12341234123412341234
```

the amount is rounded down to 8 decimals first. rounded amount: `12341234120000000000`

The payload yields the hash: `0x53bdb2747fbb47101844b4d1f5153fda18ecb4c1877504213763d4aa06f5d3b9`

The byte-array encoding of the payload passed to keccak256 is:
`0x000000000000000000000000000000000000000000000000000000000000000033441122334411223344112233441122334411223344112233441122334411220000000000000000000000006677889900112233445566778899001122334455000000000000000000000000000000000000000000000000AB44DF0BA487D000`


## Setup Steps

1. generating keyfiles
2. aggregating genesis file
3. launching network
4. deploying contracts
5. minting tokens
