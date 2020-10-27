var Bridge = artifacts.require("Bridge");
var config = require("../config.json");

module.exports = function (deployer) {
  // deployment steps
  validators = config.validators;
  let contract = deployer.deploy(Bridge, validators).then(function (x) {
    console.log("deployed-address:" + x.address);
  });
};
