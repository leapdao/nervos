// SPDX-License-Identifier: MIT
pragma solidity >=0.4.21 <0.7.2;
pragma experimental ABIEncoderV2;

/**
 * @dev Implementation of child-chain bridge.
 *
 * This contract holds the complete supply which is not in circulation.
 * For more details, check out the docs in /docs/childchain.md
 */
contract Bridge {
  // Created when the smart contract is initially deployed.
  address public deployer = msg.sender;

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
    uint8 v;
    bytes32 r;
    bytes32 s;
  }

  struct Receipt {
    bool isLock;
    address payable user;
    uint256 amount;
    bytes32 txHash;
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

  function getUnlockSignatures(bytes32 txHash) internal view returns (Sig[] memory) {
    Sig[] memory signatures  = new Sig[](validators.length * 2 / 3 + 1);
    // https://medium.com/codechain/why-n-3f-1-in-the-byzantine-fault-tolerance-system-c3ca6bab8fe9
    uint256 fillUntil = 0;
    for (uint256 i = 0; i < validators.length; i++){
      if (unlockSigs[txHash][validators[i]].v > 0) {
        signatures[fillUntil] = unlockSigs[txHash][validators[i]];
        fillUntil++;
      }
    }
    return signatures;
  }
  
  function getValidatorAddress(Receipt memory receipt, Sig memory sig) internal returns (address) {
    bytes32 receiptHash = keccak256(abi.encode(receipt.isLock, receipt.user, receipt.amount, receipt.txHash));
    // prefixed hash to mimic the behavior of eth_sign.
    bytes32 sigHash = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", receiptHash));
    
    address signer;
    (, signer) = safer_ecrecover(sigHash, sig.v, sig.r, sig.s);
    return signer;
  }

  function isMintComplete(bytes32 txHash) internal view returns (bool) {
    return lockSigs[txHash][address(0)];
  }

  function isUnlockComplete(bytes32 txHash) internal view returns (bool) {
    return unlockSigs[txHash][address(0)].v > 0;
  }

  function isValidSig(address signer) internal view returns (bool) {
    for (uint256 i = 0; i < validators.length; i++) {
      if (validators[i] == signer) {
        return true;
      }
    }
    return false;
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
    address signer) internal {
    // check the lock
    require(!isMintComplete(txHash), "Mint already executed");
    require(!lockSigs[txHash][signer], "Signature already collected");

    // Add validator signature against transaction hash
    lockSigs[txHash][signer] = true;
    emit LockSig(txHash, signer, to, amount);

    // Count all available signatures
    uint256 signerCount;
    for (uint256 v = 0; v < validators.length; v++) {
      if (lockSigs[txHash][validators[v]]) {
        signerCount++;
      }
    }

    // check for quorum
    if (signerCount > validators.length * 2 / 3) {
      // set the lock
      lockSigs[txHash][address(0)] = true;
      // how to mint?! :shrug:
      // address(uint160(to)).transfer(amount);
      to.transfer(amount);
      emit Mint(to, amount);
    }
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
    Sig memory sig,
    address signer) internal {
    // check the unlock
    require(!isUnlockComplete(txHash), "Unlock quorum already reached");
    require(unlockSigs[txHash][signer].v == 0, "Signature already collected");

    // payload
    unlockSigs[txHash][signer] = sig;
    emit UnlockSig(txHash, signer, from, amount);

    // Count all available signatures
    uint256 signerCount;
    for (uint256 u = 0; u < validators.length; u++) {
      if (unlockSigs[txHash][validators[u]].v > 0) {
        signerCount++;
      }
    }

    if (signerCount > validators.length * 2 / 3) {
      // set the lock
      unlockSigs[txHash][address(0)].v = 1;
      emit BurnQuorum(txHash, from, amount, getUnlockSignatures(txHash));
    }
  }

  /**
   * @dev Sets the list of validators that are allowed to relay events.
   */
  constructor(address[] memory _validators) {
    validators = _validators;
  }

  function collect(Receipt memory receipt, Sig memory sig) public {
    require(receipt.amount > 0, "Amount needs to be larger than zero");
    require(bytes32(0) != receipt.txHash, "txHash not equal zero");
    require(address(0) != receipt.user, "Can not receive from zero address");
    address signer = getValidatorAddress(receipt, sig); // Get validator address
    require(isValidSig(signer), "Invalid: Signature needs to be part of validator set");

    if (receipt.isLock) {
      collectLock(receipt.user, receipt.amount, receipt.txHash, signer);
    } else {
      collectUnlock(receipt.user, receipt.amount, receipt.txHash, sig, signer);
    }
  }

  /**
   * @dev On contract receiving tokens from msg.sender. 
   *
   * Emits a {Burn} event.
   */
  receive () payable external {
    if(msg.sender != deployer){
      emit Burn(msg.sender, msg.value);
    }
  }
}
