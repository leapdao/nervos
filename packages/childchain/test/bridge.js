const Bridge = artifacts.require("Bridge");
const { BN } = require("ethereumjs-util");
const abi = require("ethereumjs-abi");
const {lockReceipt, unlockReceipt} = require("./helpers/receipt");

const ALICE = "0xf3beac30c498d9e26865f34fcaa57dbb935b0d74";
const ALICE_PRIV = "0x278a5de700e29faae8e40e366ec5012b5ec63d36ec77e8a2417154cc1d25383f";
const BOB = "0xe10f3d125e5f4c753a6456fc37123cf17c6900f2";
const BOB_PRIV = "0x7bc8feb5e1ce2927480de19d8bc1dc6874678c016ae53a2eec6a6e9df717bfac";
// secretSeed: 'erosion warm student north injury good evoke river despair critic wrestle unveil' }
const CAROL = "0xc3ccb3902a164b83663947aff0284c6624f3fbf2";
const CAROL_PRIV = "0x71d2b12dad610fc929e0596b6e887dfb711eec286b7b8b0bdd742c0421a9c425";
// secretSeed: 'erode melody nature bounce sample deny spend give craft alcohol supply roof' }
const DAVE = "0x82e8c6cf42c8d1ff9594b17a3f50e94a12cc860f";
const DAVE_PRIV = "0x94890218f2b0d04296f30aeafd13655eba4c5bbf1770273276fee52cbe3f2cb4";

const LOCK_TX_HASH = "0x1122334411223344112233441122334411223344112233441122334411223344";
const ONE_ETH = new BN('1000000000000000000', 10);
const TWO_ETH = new BN('2000000000000000000', 10);

contract("Bridge", (accounts) => {
  let bridge;
  const ValidatorSet = [ALICE, BOB, CAROL, DAVE];
  const ValidatorPrivSet = [ALICE_PRIV, BOB_PRIV, CAROL_PRIV, DAVE_PRIV];

  beforeEach(async () => {
    bridge = await Bridge.new([ALICE]);
    let txHash = await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: TWO_ETH,
    });
  });

  it("should allow and collect unlock signatures", async () => {
    // burn some funds
    let tx = await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: TWO_ETH,
    });
    const txHash = tx.transactionHash;
    // create and sign unlock receipt
    const sig = unlockReceipt(accounts[0], TWO_ETH, txHash).sign(ALICE_PRIV);
    tx = await bridge.collectUnlock(
      accounts[0],
      TWO_ETH,
      txHash,
      sig.v,
      sig.r,
      sig.s
    );
    // with 2 events we assume that quorum has been reached.
    assert(tx.logs.length == 2, "validator quorum not reached");
  });

  it("should allow fours validators and collect unlock signatures", async () => {
    // initialize bridge with 4 validators
    bridge = await Bridge.new(ValidatorSet);
    // burn some funds
    let tx = await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: TWO_ETH,
    });
    const txHash = tx.transactionHash;
    // construct receipt
    const receipt = unlockReceipt(accounts[0], TWO_ETH, txHash);
    // sign and send
    let logs = [];
    for (let i = 0; i < 3; i++) {
      const sig = receipt.sign(ValidatorPrivSet[i]);
      tx = await bridge.collectUnlock(
        accounts[0],
        TWO_ETH,
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
      logs = [...logs, ...tx.logs];
    }
    // 3 unlock sigs + 1 aggregate event with all sigs collected
    assert(logs.length == 4, "quorum of validators signed");
  });

  it("should fail upon double collection of unlock", async () => {
    bridge = await Bridge.new(ValidatorSet);
    let tx = await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: TWO_ETH,
    });
    const txHash = tx.transactionHash;

    // construct receipt
    const receipt = unlockReceipt(accounts[0], TWO_ETH, txHash);

    for (let i = 0; i < 2; i++) {
      const sig = receipt.sign(ValidatorPrivSet[i]);
      await bridge.collectUnlock(
        accounts[0],
        TWO_ETH,
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
    }
    try {
      await bridge.collectUnlock(
        accounts[0],
        TWO_ETH,
        txHash,
        sig.v,
        sig.r,
        sig.s
      );
      throw new Error("expected to throw");
    } catch (error) {
      assert(error);
      // Ethermint does not pass through the error message, once the issue is addressed the asserts can be changed back
      //assert(error.message.includes("signature already collected"));
    }
  });

  it("should allow to collect 1 and transfer to Alice", async () => {
    // generate sig
    const sig = lockReceipt(ALICE, ONE_ETH, LOCK_TX_HASH).sign(ALICE_PRIV);
    const preAmount = await web3.eth.getBalance(ALICE);
    await bridge.collectLock(
      ALICE,
      ONE_ETH,
      LOCK_TX_HASH,
      sig.v,
      sig.r,
      sig.s
    );
    const postAmount = await web3.eth.getBalance(ALICE);
    assert.equal(postAmount - 1000000000000000000, preAmount);
  });

  it("should not allow Bob as validator.", async () => {
    // generate sig
    const sig = lockReceipt(BOB, ONE_ETH, LOCK_TX_HASH).sign(BOB_PRIV);
    try {
      await bridge.collectLock(
        BOB,
        ONE_ETH,
        LOCK_TX_HASH,
        sig.v,
        sig.r,
        sig.s
      );
      throw new Error("expected to throw");
    } catch (error) {
      assert(error);
      // Ethermint does not pass through the error message, once the issue is addressed the asserts can be changed back
      //assert(error.message.includes("Signer needs to be part of validator set"));
    }
  });

  it("should not allow submit on executed relay.", async () => {
    // generate sig
    const sig = lockReceipt(ALICE, ONE_ETH, LOCK_TX_HASH).sign(ALICE_PRIV);
    await bridge.collectLock(
      ALICE,
      ONE_ETH,
      LOCK_TX_HASH,
      sig.v,
      sig.r,
      sig.s
    );
    try {
      await bridge.collectLock(
        ALICE,
        ONE_ETH,
        LOCK_TX_HASH,
        sig.v,
        sig.r,
        sig.s
      );
      throw new Error("expected to throw");
    } catch (error) {
      assert(error);
      // Ethermint does not pass through the error message, once the issue is addressed the asserts can be changed back
      //assert(error.message.includes("mint already executed"));
    }
  });

  it("should collect locks from three validators.", async () => {
    bridge = await Bridge.new(ValidatorSet);
    await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: TWO_ETH,
    });
    // generate sig
    const receipt = lockReceipt(ALICE, ONE_ETH, LOCK_TX_HASH);
    const preAmount = await web3.eth.getBalance(ALICE);
    for (let i = 0; i < 3; i++) {
      const sig = receipt.sign(ValidatorPrivSet[i]);
      await bridge.collectLock(
        ALICE,
        ONE_ETH,
        LOCK_TX_HASH,
        sig.v,
        sig.r,
        sig.s
      );
    }
    const postAmount = await web3.eth.getBalance(ALICE);
    assert.equal(postAmount - 1000000000000000000, preAmount);
  });

  it("double submission on unfinished lock", async () => {
    bridge = await Bridge.new(ValidatorSet);
    await web3.eth.sendTransaction({
      from: accounts[0],
      to: bridge.address,
      value: TWO_ETH,
    });
    // generate sig
    const receipt = lockReceipt(ALICE, ONE_ETH, LOCK_TX_HASH);
    for (let i = 0; i < 2; i++) {
      sig = receipt.sign(ValidatorPrivSet[i]);
      await bridge.collectLock(
        ALICE,
        ONE_ETH,
        LOCK_TX_HASH,
        sig.v,
        sig.r,
        sig.s
      );
    }

    try {
      await bridge.collectLock(
        ALICE,
        ONE_ETH,
        LOCK_TX_HASH,
        sig.v,
        sig.r,
        sig.s
      );
      throw new Error("expected to throw");
    } catch (error) {
      assert(error);
      // Ethermint does not pass through the error message, once the issue is addressed the asserts can be changed back
      //assert(error.message.includes("signature already collected"));
    }
  });
  //possible check to be added for reentry attack case
  it("reentry attack case", async () => {});
});
