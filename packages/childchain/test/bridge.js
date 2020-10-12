const Bridge = artifacts.require("Bridge");
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

// No need for real transaction unless asserting value changes;
const TX_HASH = "0x1122334411223344112233441122334411223344112233441122334411223344";

const ONE_ETH = web3.utils.numberToHex('1000000000000000000');
const TWO_ETH = web3.utils.numberToHex('2000000000000000000');

const ValidatorSet = [ALICE_PRIV, CAROL_PRIV, DAVE_PRIV];

contract("Bridge", (accounts) => {
  let bridge;
  let unLockObject;
  let lockObject;

  const EXISTING_ACCOUNT = accounts[0];

  beforeEach(async () => {
    bridge = await Bridge.new([ALICE, CAROL, DAVE]);
    unLockObject = unlockReceipt(EXISTING_ACCOUNT, TWO_ETH, TX_HASH, web3);
    lockObject = lockReceipt(EXISTING_ACCOUNT, TWO_ETH, TX_HASH, web3);
  });

  it("should allow and collect unlock signatures", async () => {
    const [receipt, sig] = unLockObject.getPayload(ALICE_PRIV);
    tx = await bridge.collect(receipt, sig);
    
    // check for unlock event.    
    assert(tx.logs.length == 1, "validator quorum not reached");
    assert(tx.logs[0].event == "UnlockSig", "Unlock event present");
  });

  it("should collect unlocks quorom and increase bridge balance", async () => {
    let tx;
    for (let x = 0; x < ValidatorSet.length ; x++) {
      const [receipt, sig] = unLockObject.getPayload(ValidatorSet[x]);
      tx = await bridge.collect(receipt, sig);
    }
    // TODO:: new test check user pre & post bridge balance 
    // last transaction should have BurnQuorum event
    assert(tx.logs.length, 2); // Both unlock & burn events for last signer
    assert(tx.logs[1].event, "BurnQuorum");

    const postBridgeAmount = web3.utils.numberToHex(await web3.eth.getBalance(bridge.address));
    assert.equal(postBridgeAmount, TWO_ETH); //Should be 2 ETH
  });

  it("should fail on double submission of unlock", async () => {
    const [receipt, sig] = unLockObject.getPayload(ALICE_PRIV);
    await bridge.collect(receipt, sig);

    try {
      await bridge.collect(receipt, sig);
    } catch (error) {
      // Ethermint does not pass through the error message, once the issue is addressed the asserts can be changed back
      //assert(error.message.includes("signature already collected"));
      assert(error); // Generic error
    }
  });

  it("should not allow Bob as validator", async () => {
    const [receipt, sig] = unLockObject.getPayload(BOB_PRIV);

    try {
      await bridge.collect(receipt, sig);
    } catch (error) {
      // Ethermint does not pass through the error message, once the issue is addressed the asserts can be changed back
      //assert(error.message.includes("Signer needs to be part of validator set"));
      assert(error); // Generic error
    }
  });

  it("should not allow submit on executed relay", async () => {
    // submit txHash for 3 validators 
    for (let x = 0; x < ValidatorSet.length; x++) {
      const [receipt, sig] = unLockObject.getPayload(ValidatorSet[x]);
      await bridge.collect(receipt, sig); 
    }
    
    // try a 4th submission
    try {
      // Submit after all other validators
      const [receipt, sig] = unLockObject.getPayload(ValidatorSet[0]); 
      await bridge.collect(receipt, sig);
    } catch (error) {
      // Ethermint does not pass through the error message, once the issue is addressed the asserts can be changed back
      //assert(error.message.includes("Signer needs to be part of validator set"));
      assert(error); // Generic error
    }
  });

  it("should allow collect of lock", async () => {
    const [receipt, sig] = lockObject.getPayload(ALICE_PRIV);
    tx = await bridge.collect(receipt, sig);    
    // check for locksig event.    
    assert(tx.logs.length == 1, "validator quorum not reached");
    assert(tx.logs[0].event == "LockSig", "Unlock event present");
  });

  it("should collect locks quorom and mint tokens to user", async () => {

    // Transfer 2 ETH to wallet
    let tx = await web3.eth.sendTransaction({
      from: EXISTING_ACCOUNT, // validator address has tokens
      to: bridge.address,
      value: TWO_ETH,
      gasPrice: '20000000000'
    });

    const NEW_ACCOUNT = await web3.eth.personal.newAccount('!@superpassword');

    // generate sig
    const preAmount = await web3.eth.getBalance(NEW_ACCOUNT);
    const preBridgeAmount = await web3.eth.getBalance(bridge.address); // TWO ETH

    // use 1 ETH cause bridge has 2 ETH
    lockObject = lockReceipt(NEW_ACCOUNT, ONE_ETH, TX_HASH, web3);
    for (let x = 0; x < ValidatorSet.length; x++) {
      const [receipt, sig] = lockObject.getPayload(ValidatorSet[x]);
      tx = await bridge.collect(receipt, sig); 
    }

    // last transaction should have Mint event
    assert(tx.logs.length == 2);
    assert(tx.logs[1].event == "Mint");

    const postBridgeAmount = await web3.eth.getBalance(bridge.address);
    const postAmount = await web3.eth.getBalance(NEW_ACCOUNT);
    assert.equal(+postBridgeAmount + 1000000000000000000, preBridgeAmount);
    assert.equal(postAmount, +preAmount + 1000000000000000000);    
    
  });

  it("should fail on double submission of lock", async () => {
    const [receipt, sig] = lockObject.getPayload(ALICE_PRIV);
    await bridge.collect(receipt, sig);   

    try {
      await bridge.collect(receipt, sig);
    } catch (error) {
      // Ethermint does not pass through the error message, once the issue is addressed the asserts can be changed back
      //assert(error.message.includes("signature already collected"));
      assert(error); // Generic error
    }
  });
  //possible check to be added for reentry attack case
  it("reentry attack case", async () => {});
});
