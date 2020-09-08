/**
 * Copyright (c) 2018-present, Leap DAO (leapdao.org)
 *
 * This source code is licensed under the Mozilla Public License, version 2,
 * found in the LICENSE file in the root directory of this source tree.
 */

const Signer = require("./signer");

function lockReceipt(address, amount, hash) {
  // todo: check inputs
	return new Signer(true, address, amount, hash);
}

function unlockReceipt(address, amount, hash) {
  return new Signer(false, address, amount, hash);
}

module.exports = {
  lockReceipt,
  unlockReceipt,
};