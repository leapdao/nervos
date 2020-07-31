const Bridge = artifacts.require("Bridge");
const ethUtil = require('ethereumjs-util');

contract('Bridge', accounts => {
	const alice = '0xf3beac30c498d9e26865f34fcaa57dbb935b0d74';
	const txHash = '0x1122334411223344112233441122334411223344112233441122334411223344';
	const validatorPriv = '0x278a5de700e29faae8e40e366ec5012b5ec63d36ec77e8a2417154cc1d25383f';
	let bridge;

	before(async() => {
		bridge = await Bridge.new([alice]);
	});

	it('should allow to collect 1', async() => {

		// generate sig
		const buf = Buffer.alloc(96);
		Buffer.from(alice.replace('0x', ''), 'hex').copy(buf, 0, 0, 20);
		Buffer.from('0000000000000000000000000000000000000000000000000DE0B6B3A7640000', 'hex').copy(buf, 32, 0, 32);
		Buffer.from(txHash.replace('0x', ''), 'hex').copy(buf, 64, 0, 32);
		const sigHash = ethUtil.keccak256(buf);
		console.log(sigHash);
    const sig = ethUtil.ecsign(Buffer.from(validatorPriv.replace('0x', ''), 'hex'), sigHash);
    console.log(sig);
		await bridge.collectLock(alice, '1000000000000000000', txHash, sig.v, sig.r, sig.s);

	});
})