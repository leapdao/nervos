const Bridge = artifacts.require("Bridge");
const ethUtil = require("ethereumjs-util");
var abi = require("ethereumjs-abi");

contract("Bridge", (accounts) => {
  const alice = "0xf3beac30c498d9e26865f34fcaa57dbb935b0d74";
  const txHash =
    "0x1122334411223344112233441122334411223344112233441122334411223344";
  const validatorPriv =
    "0x278a5de700e29faae8e40e366ec5012b5ec63d36ec77e8a2417154cc1d25383f";
  let bridge;
  const bob = "0xe10f3d125e5f4c753a6456fc37123cf17c6900f2";
  const bobPriv =
    "0x7bc8feb5e1ce2927480de19d8bc1dc6874678c016ae53a2eec6a6e9df717bfac";

  // secretSeed: 'erosion warm student north injury good evoke river despair critic wrestle unveil' }
  const P3_ADDR = "0xc3ccb3902a164b83663947aff0284c6624f3fbf2";
  const P3_PRIV =
    "0x71d2b12dad610fc929e0596b6e887dfb711eec286b7b8b0bdd742c0421a9c425";

  // secretSeed: 'erode melody nature bounce sample deny spend give craft alcohol supply roof' }
  const P4_ADDR = "0x82e8c6cf42c8d1ff9594b17a3f50e94a12cc860f";
  const P4_PRIV =
    "0x94890218f2b0d04296f30aeafd13655eba4c5bbf1770273276fee52cbe3f2cb4";

  const ValidatorSet = [alice, bob, P3_ADDR, P4_ADDR];
  const ValidatorPrivSet = [validatorPriv, bobPriv, P3_PRIV, P4_PRIV];

  beforeEach(async () => {
    bridge = await Bridge.new([alice]);
    let txHash = await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: "2000000000000000000",
    });
  });

  /*
  it("should not crash Ethermint", async () => {
    let unlockSigs = await bridge.getPastEvents(
      "Burn",
      { txHash: txHash },
      { fromBlock: 0, toBlock: "latest" }
    );
    bridge = await Bridge.new(ValidatorSet);
    bridge = await Bridge.new(ValidatorSet);
  });
*/
  it("should allow and collect unlock signatures", async () => {
    let txHash = await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: "2000000000000000000",
    });
    txHash = txHash.transactionHash;
    let payload = abi.rawEncode(
      ["address", "uint256", "bytes32"],
      [accounts[0], "2000000000000000000", txHash]
    );
    const sigHash = ethUtil.keccak256(payload);
    const sig = ethUtil.ecsign(
      sigHash,
      Buffer.from(validatorPriv.replace("0x", ""), "hex")
    );
    let tx = await bridge.collectUnlock(
      accounts[0],
      "2000000000000000000",
      txHash,
      sig.v,
      sig.r,
      sig.s
    );
    assert(tx.logs.length == 2);
  });

  it("should allow fours validators and collect unlock signatures", async () => {
    bridge = await Bridge.new(ValidatorSet);
    let tx = await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: "2000000000000000000",
    });
    let txHash = tx.transactionHash;
    let payload = abi.rawEncode(
      ["address", "uint256", "bytes32"],
      [accounts[0], "2000000000000000000", txHash]
    );
    const sigHash = ethUtil.keccak256(payload);
    let sigs = [];
    for (let i = 0; i < 3; i++) {
      let sig = ethUtil.ecsign(
        sigHash,
        Buffer.from(ValidatorPrivSet[i].replace("0x", ""), "hex")
      );
      tx = await bridge.collectUnlock(
        accounts[0],
        "2000000000000000000",
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
      sigs = [...sigs, ...tx.logs];
    }
    assert(sigs.length == 4, "all validators signed");
  });

  it("should fail upon double collection of unlock", async () => {
    bridge = await Bridge.new(ValidatorSet);
    let tx = await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: "2000000000000000000",
    });
    let txHash = tx.transactionHash;
    let payload = abi.rawEncode(
      ["address", "uint256", "bytes32"],
      [accounts[0], "2000000000000000000", txHash]
    );
    let sig;
    const sigHash = ethUtil.keccak256(payload);
    for (let i = 0; i < 2; i++) {
      sig = ethUtil.ecsign(
        sigHash,
        Buffer.from(ValidatorPrivSet[i].replace("0x", ""), "hex")
      );
      await bridge.collectUnlock(
        accounts[0],
        "2000000000000000000",
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
    }
    try {
      await bridge.collectUnlock(
        accounts[0],
        "2000000000000000000",
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
      throw new Error("expected to throw");
    } catch (error) {
      assert(error);
      //assert(error.message.includes("signature already collected"));
    }
  });

  it("should allow to collect 1 and transfer to Alice", async () => {
    // generate sig
    let payload = abi.rawEncode(
      ["address", "uint256", "bytes32"],
      [alice, "1000000000000000000", txHash]
    );
    const sigHash = ethUtil.keccak256(payload);
    const sig = ethUtil.ecsign(
      sigHash,
      Buffer.from(validatorPriv.replace("0x", ""), "hex")
    );
    const preAmount = await web3.eth.getBalance(alice);
    await bridge.collectLock(
      alice,
      "1000000000000000000",
      txHash,
      sig.v,
      sig.r,
      sig.s
    );
    const postAmount = await web3.eth.getBalance(alice);
    assert.equal(postAmount - 1000000000000000000, preAmount);
  });

  it("should not allow Bob as validator.", async () => {
    // generate sig
    let payload = abi.rawEncode(
      ["address", "uint256", "bytes32"],
      [bob, "1000000000000000000", txHash]
    );
    const sigHash = ethUtil.keccak256(payload);
    const sig = ethUtil.ecsign(
      sigHash,
      Buffer.from(bobPriv.replace("0x", ""), "hex")
    );
    try {
      await bridge.collectLock(
        bob,
        "1000000000000000000",
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
      throw new Error("expected to throw");
    } catch (error) {
      assert(error);
      //assert(error.message.includes("Signer needs to be part of validator set"));
    }
  });

  it("should not allow submit on executed relay.", async () => {
    // generate sig
    let payload = abi.rawEncode(
      ["address", "uint256", "bytes32"],
      [alice, "1000000000000000000", txHash]
    );
    const sigHash = ethUtil.keccak256(payload);
    const sig = ethUtil.ecsign(
      sigHash,
      Buffer.from(validatorPriv.replace("0x", ""), "hex")
    );
    await bridge.collectLock(
      alice,
      "1000000000000000000",
      txHash,
      sig.v,
      sig.r,
      sig.s
    );
    try {
      await bridge.collectLock(
        alice,
        "1000000000000000000",
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
      throw new Error("expected to throw");
    } catch (error) {
      assert(error);
      //assert(error.message.includes("mint already executed"));
    }
  });

  it("should collect locks from three validators.", async () => {
    bridge = await Bridge.new(ValidatorSet);
    await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: "2000000000000000000",
    });
    // generate sig
    let payload = abi.rawEncode(
      ["address", "uint256", "bytes32"],
      [alice, "1000000000000000000", txHash]
    );
    const sigHash = ethUtil.keccak256(payload);
    const preAmount = await web3.eth.getBalance(alice);
    for (let i = 0; i < 3; i++) {
      let sig = ethUtil.ecsign(
        sigHash,
        Buffer.from(ValidatorPrivSet[i].replace("0x", ""), "hex")
      );
      await bridge.collectLock(
        alice,
        "1000000000000000000",
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
    }
    const postAmount = await web3.eth.getBalance(alice);
    assert.equal(postAmount - 1000000000000000000, preAmount);
  });

  it("double submission on unfinished lock", async () => {
    bridge = await Bridge.new(ValidatorSet);
    await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: "2000000000000000000",
    });
    // generate sig
    let payload = abi.rawEncode(
      ["address", "uint256", "bytes32"],
      [alice, "1000000000000000000", txHash]
    );
    const sigHash = ethUtil.keccak256(payload);
    let sig;
    for (let i = 0; i < 2; i++) {
      sig = ethUtil.ecsign(
        sigHash,
        Buffer.from(ValidatorPrivSet[i].replace("0x", ""), "hex")
      );
      await bridge.collectLock(
        alice,
        "1000000000000000000",
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
    }

    try {
      await bridge.collectLock(
        alice,
        "1000000000000000000",
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
      throw new Error("expected to throw");
    } catch (error) {
      assert(error);
      //assert(error.message.includes("signature already collected"));
    }
  });

  it("reentry attack case", async () => {});
});
