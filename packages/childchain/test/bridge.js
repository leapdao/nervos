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

  beforeEach(async () => {
    bridge = await Bridge.new([alice]);
    await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: "2000000000000000000",
    });
    await web3.eth.sendTransaction({
      from: accounts[0],
      to: bob,
      value: "2000000000000000000",
    });
  });

  it("should allow and collect unlock signatures", async () => {
    let bobAccount = await web3.eth.accounts.privateKeyToAccount(bobPriv);
    let txHash = await web3.eth.sendTransaction({
      from: bob,
      to: bridge.address,
      value: "1000000000000000000",
    });
    let burnEvents = await bridge.Burn(
      { sender: bob },
      { fromBlock: 0, toBlock: "latest" }
    );
    let txHash = burnEvents[0].transactionHash;
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
    await bridge.collectUnlock(
      alice,
      "1000000000000000000",
      txHash,
      sig.v,
      sig.r,
      sig.s
    );
    const postAmount = await web3.eth.getBalance(alice);
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
      assert(
        error.message.includes("Signer needs to be part of validator set")
      );
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
      assert(error.message.includes("mint already executed"));
    }
  });
});
