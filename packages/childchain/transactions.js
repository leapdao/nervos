var Web3 = require("web3");
var web3 = new Web3("http://localhost:8545");
var BN = web3.utils.BN;
var config = require("./config.json");
let myArgs = process.argv.slice(3);
const privateKey = myArgs[0];
const bridgeAddress = myArgs[1];
let account = web3.eth.accounts.privateKeyToAccount(privateKey);
const validators = config.validators;
(async () => {
  //funding validators
  for (let i = 0; i < validators.length; i++) {
    await web3.eth.sendTransaction({
      from: account.address,
      to: validators[i],
      value: config.validatorStake,
    });
  }
  //funding bridge
  await web3.eth.sendTransaction({
    from: account.address,
    to: bridgeAddress,
    value: "10000000000000000000000000000",
  });
  //sending rest of the balance to zero address
  let totalBalance = new BN(await web3.eth.getBalance(account.address));
  totalBalance = totalBalance.sub(new BN("21000"));
  console.log(
    await web3.eth.sendTransaction({
      from: account.address,
      to: "0x0000000000000000000000000000000000000000",
      value: totalBalance,
    })
  );
})();
