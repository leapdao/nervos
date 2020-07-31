// SPDX-License-Identifier: MIT
pragma solidity >=0.4.21 <0.7.0;

contract Bridge {

  event LockSig(bytes32 indexed txHash, address indexed validator, address to, uint256 amount);

  address[] public validators;
  mapping(bytes32 => mapping(address => bool)) lockSigs;
  // mapping(bytes32 => address[]) lockSigs;

  constructor(address[] memory _validators) public {
    validators = _validators;
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
    require(!lockSigs[txHash][to], "mint already executed");
    bytes32 sigHash = keccak256(abi.encode(to, amount, txHash));
    address signer;
    (, signer) = safer_ecrecover(sigHash, v, r, s);
    uint256 signerCount = 0;
    for (v = 0; v < validators.length; v++) {
      if (validators[v] == signer) {
        // payload
        lockSigs[txHash][signer] = true;
        emit LockSig(txHash, signer, to, amount);
      }
      if (lockSigs[txHash][signer]) {
        signerCount++;
      }
    }
    if (signerCount > validators.length * 2 / 3) {
      // how to mint?! :shrug:
      //to.transfer(amount);
      // set the lock
      lockSigs[txHash][to] = true;
    }
  }
}
