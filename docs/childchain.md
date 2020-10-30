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
isLock: false,
user: '0x6677889900112233445566778899001122334455',
amount: '12341234123412341234',
txHash: '0x3344112233441122334411223344112233441122334411223344112233441122',
```

the amount is rounded down to 8 decimals first. rounded amount: `12341234120000000000`

The payload yields the hash: `0x6cdbcd8791511382edce33435ba9a3fde6836a9b9b6c17cd84b93ff28639d021`

The byte-array encoding of the payload passed to keccak256 is:
`0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000006677889900112233445566778899001122334455000000000000000000000000000000000000000000000000ab44df0c6fec01f23344112233441122334411223344112233441122334411223344112233441122`

A signature by `0xf3beac30c498d9e26865f34fcaa57dbb935b0d74` resolves to:
```
v: '0x1c',
r: '0x1432a626402e14f716905f400d67d0cc611d5d7e5d3cf4495424578ba54d6cbd',
s: '0x0a483431592c222444fbd6171873a3ba9580ddf279ae2885e4586d36d9e99c4c'
```

## Contract Interface

The contract has 2 major functions:
- collect lock receipts and mint funds: The lock collection follows a transaction on the parent chain that locks tokens into the parent bridge. 
- allow to burn funds and collect unlock receipts: The collection of unlock sigs is mostly a convenience functions for validators, to avoid off-chain coordination.

### Collect Lock Receipts

Any validator can call the `collectLock()` function. It's interface is as follows:

```
  function collectLock(
    address payable to,
    uint256 amount,
    bytes32 txHash,
    uint8 v,
    bytes32 r,
    bytes32 s) public {	
  }
```

The function aggregates signature of lock receipts in storage of contract. The function emits a `LockSig event each time a new valid signature is collected. The event is specified as:
```
  event LockSig(bytes32 indexed txHash, address indexed validator, address to, uint256 amount);
```

Once the quorum is reached, `amount` tokens are released to receiver (`to`). A `Mint` event is emitted with the following structure:

```
  event Mint(address indexed receiver, uint256 value);
```

A lock is set in the contract based on the `txHash` to prevent doubleminting.

### Collect Burn Receipts

The process begins with the call of the burn function, which moves `msg.value` tokens from `msg.sender` into non-circulating supply, as defined here:

```
  function () payable external {
  }
```

The function emits a Burn event as follows:
```
  event Burn(address indexed sender, uint256 value);
```

Once funds have been burned the contract has a helper function to coordinate signature aggregation. The function is defined as:
```
  function collectUnlock(
    address from,
    uint256 amount,
    bytes32 txHash,
    uint8 v,
    bytes32 r,
    bytes32 s) public {
  }
```

An `UnlockSig` is emitted every time with the following data:
```
  event UnlockSig(bytes32 indexed txHash, address indexed validator, address from, uint256 amount);
```

Once the quorum has been reached an aggregate event is emitted with all signatures included:

```
  event BurnQuorum(bytes32 indexed txHash, address indexed from, uint256 amount, Sig[] signatures);
```

The contained data can be relayed to the parent bridge to unlock funds to the address of the burner. Further signatures can not be collected under the same `txHash`. The `txHash` should be used as a lock in the parent-chain bridge to prevent double-payments.


## Setup Steps

Ideally all steps will be sumarized in single script. The address of the parent-bridge contract should be taken as an input. The validators should have locked small amount of CKB in the bridge already, so that child-chain balances can be minted for gas payments.

1. generating keyfiles
2. aggregating genesis file
3. launching network
4. deploying contracts
5. minting tokens
