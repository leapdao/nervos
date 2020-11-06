const ethers = require("ethers");
async function main() {

  let provider = ethers.getDefaultProvider('ropsten');
  
  // Create a wallet to sign the message with
  let privateKey = '0x278a5de700e29faae8e40e366ec5012b5ec63d36ec77e8a2417154cc1d25383f';
  let wallet = new ethers.Wallet(privateKey);

  console.log(wallet._signingKey().publicKey);

  let message = ethers.utils.arrayify("0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000006677889900112233445566778899001122334455000000000000000000000000000000000000000000000000000000000000000a3344112233441122334411223344112233441122334411223344112233441122");
  // let hash = ethers.utils.keccak256(message);
  // Sign the string message
  let flatSig = await wallet.signMessage(message);
  console.log(flatSig);
  let sig = ethers.utils.splitSignature(flatSig);
  console.log(sig);
}

main();
