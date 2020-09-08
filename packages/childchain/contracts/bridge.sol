// SPDX-License-Identifier: MIT
pragma solidity >=0.4.21 <0.7.0;
pragma experimental ABIEncoderV2;

/**
 * @dev Implementation of child-chain bridge.
 *
 * This contract holds the complete supply which is not in circulation.
 * For more details, check out the docs in /docs/childchain.md
 */
contract Bridge {

  /**
   * @dev Emitted when a validator (`validator`) relays a lock event with a recipient (`to`) 
   * and amount (`amount`). 
   *
   * Note that `value` must NOT be zero.
   */
  event LockSig(bytes32 indexed txHash, address indexed validator, address to, uint256 amount);
  /**
   * @dev Emitted when a quorum of signatures by validators has been reached. 
   * Native tokens of amount (`amount`) are transfered to recipient (`receiver`).
   */
  event Mint(address indexed receiver, uint256 value);
  /**
   * @dev Emitted when any token holder (`sender`) is burning an amount of tokens (`value`)
   *
   * Note that `value` must NOT be zero.
   */
  event Burn(address indexed sender, uint256 value);
  /**
   * @dev Emitted when a validator submits a signature over data of a burn event.
   */
  event UnlockSig(bytes32 indexed txHash, address indexed validator, address from, uint256 amount);
  /**
   * @dev Emitted when a quorum of validator have submited signatures over data of a burn event.
   */
  event BurnQuorum(bytes32 indexed txHash, address indexed from, uint256 amount, Sig[] signatures);

  address[] public validators;
  mapping(bytes32 => mapping(address => bool)) lockSigs;

  struct Sig {
    bool complete;
    uint8 v;
    bytes32 r;
    bytes32 s;
  }

  mapping(bytes32 => mapping(address => Sig)) unlockSigs;

  function safer_ecrecover(bytes32 hash, uint8 v, bytes32 r, bytes32 s) internal returns (bool, address) {
    // We do our own memory management here. Solidity uses memory offset
    // 0x40 to store the current end of memory. We write past it (as
    // writes are memory extensions), but don't update the offset so
    // Solidity will reuse it. The memory used here is only needed for
    // this context.

    // FIXME: inline assembly can't access return values
    bool ret;
    address addr;

    assembly {
      let size := mload(0x40)
      mstore(size, hash)
      mstore(add(size, 32), v)
      mstore(add(size, 64), r)
      mstore(add(size, 96), s)

      // NOTE: we can reuse the request memory because we deal with
      //       the return code
      ret := call(3000, 1, 0, size, 128, size, 32)
      addr := mload(size)
    }

    return (ret, addr);
  }

  function isMintComplete(bytes32 txHash) internal view returns (bool) {
    return lockSigs[txHash][address(0)];
  }

  function isUnlockComplete(bytes32 txHash) internal view returns (bool) {
    return unlockSigs[txHash][address(0)].complete;
  }

  /**
   * @dev Sets the list of validators that are allowed to relay events.
   */
  constructor(address[] memory _validators) public {
    validators = _validators;
  }


  /**
   * @dev aggregates signature of lock receipts in storage of contract.
   * once the quorum is reached, `amount` tokens are released to receiver (`to`).
   *
   * Emits a {LockSig} event.
   * Emits a {Mint} event.
   */
  function collectLock(
    address payable to,
    uint256 amount,
    bytes32 txHash,
    uint8 v,
    bytes32 r,
    bytes32 s) public {
    // check the lock
    require(!isMintComplete(txHash), "mint already executed");
    require(amount > 0, "amount needs to be larger than zero");
    bytes32 sigHash = keccak256(abi.encode(true, to, amount, txHash));
    address signer;
    (, signer) = safer_ecrecover(sigHash, v, r, s);
    uint256 signerCount = 0;
    require(!lockSigs[txHash][signer], "signature already collected");
    for (v = 0; v < validators.length; v++) {
      // add the new signature
      if (validators[v] == signer) {
        // payload
        lockSigs[txHash][signer] = true;
        emit LockSig(txHash, signer, to, amount);
      }
      // count all available signatures
      if (lockSigs[txHash][validators[v]]) {
        signerCount++;
      }
    }
    // check for quorum
    if (signerCount > validators.length * 2 / 3) {
      // set the lock
      lockSigs[txHash][address(0)] = true;
      // how to mint?! :shrug:
      to.transfer(amount);
      emit Mint(to, amount);
    }
    require(lockSigs[txHash][signer] == true, "Signer needs to be part of validator set");
  }

  /**
   * @dev moves `msg.value` tokens from `msg.sender` account into non-circulating supply.
   *
   * Emits a {Burn} event.
   */
  function () payable external {
    emit Burn(msg.sender, msg.value);
  }

  /**
   * @dev aggregates signature of unlock receipts in storage of contract.
   * Once the quorum is reached, a aggregate event is emmited with all signatures.
   *
   * Emits a {UnlockSig} event.
   * Emits a {BurnQuorum} event.
   */
  function collectUnlock(
    address from,
    uint256 amount,
    bytes32 txHash,
    uint8 v,
    bytes32 r,
    bytes32 s) public {
    // check the unlock
    require(!isUnlockComplete(txHash), "unlock quorum already reached");
    require(amount > 0, "amount needs to be larger than zero");
    require(address(0) != from, "can not receive from zero address");
    require(bytes32(0) != txHash, "txHash not equal zero");
    bytes32 sigHash = keccak256(abi.encode(false, from, amount, txHash));
    address signer;
    (, signer) = safer_ecrecover(sigHash, v, r, s);
    uint256 signerCount = 0;
    require(!unlockSigs[txHash][signer].complete, "signature already collected");
    for (uint256 i = 0; i < validators.length; i++) {
      if (validators[i] == signer) {
        // payload
        unlockSigs[txHash][signer] = Sig({
          complete: true,
          v: v,
          r: r,
          s: s
        });
        emit UnlockSig(txHash, signer, from, amount);
      }
      if (unlockSigs[txHash][validators[i]].v > 0) {
        signerCount++;
      }
    }
    if (signerCount > validators.length * 2 / 3) {
      // how to mint?! :shrug:
      // set the lock
      Sig[] memory signatures  = new Sig[](validators.length * 2 / 3 + 1);
      // https://medium.com/codechain/why-n-3f-1-in-the-byzantine-fault-tolerance-system-c3ca6bab8fe9
      uint256 fillUntil = 0;
      for (uint256 i = 0; i < validators.length; i++){
        if (unlockSigs[txHash][validators[i]].v > 0) {
          signatures[fillUntil] = unlockSigs[txHash][validators[i]];
          fillUntil++;
        }
      }
      unlockSigs[txHash][address(0)].complete = true;
      emit BurnQuorum(txHash, from, amount, signatures);
    }
    require(unlockSigs[txHash][signer].complete, "Signer needs to be part of validator set");
  }
}