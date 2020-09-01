// SPDX-License-Identifier: MIT
pragma solidity >=0.4.21 <0.7.0;
pragma experimental ABIEncoderV2;

contract Bridge {

  event LockSig(bytes32 indexed txHash, address indexed validator, address to, uint256 amount);
  event Burn(address indexed sender, uint256 value);
  event UnlockSig(bytes32 indexed txHash, address indexed validator, address from, uint256 amount);
  event BurnQuorum(bytes32 indexed txHash, address indexed from, uint256 amount, Sig[] signatures);

  address[] public validators;
  mapping(bytes32 => mapping(address => bool)) lockSigs;
  // mapping(bytes32 => address[]) lockSigs;

  struct Sig {
    bool complete;
    uint8 v;
    bytes32 r;
    bytes32 s;
  }

  mapping(bytes32 => mapping(address => Sig)) unlockSigs;

  constructor(address[] memory _validators) public {
    validators = _validators;
  }

  function () payable external {
      for (uint256 v = 0; v < validators.length; v++) {
        if (validators[v] == msg.sender) {
          return;
        }
      }
      emit Burn(msg.sender, msg.value);
    }

    function collectUnlock(
      address from,
      uint256 amount,
      bytes32 txHash,
      uint8 v,
      bytes32 r,
      bytes32 s) public {
      // check the unlock
      require(!unlockSigs[txHash][address(0)].complete, "burn already completed");
      require(amount > 0, "amount needs to be larger than zero");
      require(address(0) != from, "can not receive from zero address");
      require(bytes32(0) != txHash, "txHash not equal zero");
      bytes32 sigHash = keccak256(abi.encode(from, amount, txHash));
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
      require(signerCount > 0, "Signer needs to be part of validator set");
    }

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

  function collectLock(
    address payable to,
    uint256 amount,
    bytes32 txHash,
    uint8 v,
    bytes32 r,
    bytes32 s) public {
    // check the lock
    require(!lockSigs[txHash][address(0)], "mint already executed");
    require(amount > 0, "amount needs to be larger than zero");
    bytes32 sigHash = keccak256(abi.encode(to, amount, txHash));
    address signer;
    (, signer) = safer_ecrecover(sigHash, v, r, s);
    uint256 signerCount = 0;
    require(!lockSigs[txHash][signer], "signature already collected");
    for (v = 0; v < validators.length; v++) {
      if (validators[v] == signer) {
        // payload
        lockSigs[txHash][signer] = true;
        emit LockSig(txHash, signer, to, amount);
      }
      if (lockSigs[txHash][validators[v]]) {
        signerCount++;
      }
    }
    if (signerCount > validators.length * 2 / 3) {
      // set the lock
      lockSigs[txHash][address(0)] = true;
      // how to mint?! :shrug:
      to.transfer(amount);
    }
    require(signerCount > 0, "Signer needs to be part of validator set");
  }
}
