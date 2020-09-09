# Running the node

To start the node make sure to have ethermint (v0.1.0 or higher) installed and have your GO Path exposed:

```
git clone https://github.com/ChainSafe/ethermint
cd ethermint
make install
```

# Run the tests

To compile the contracts run `truffle build`, to run the tests run `truffle test`.

# Start-up script

The underlying bash-Script `childchain.sh` starts the Ethermint chain, initializes a genesis account as well as starting a RPC server. To stop the chain the command `pkill ethermint` should be used or analogous command.

Requirements to run this script are the installation of Ethermint, truffle, node.js, perl and sed.

After the Ethermint chain is started, the genesis account is used to deploy the Bridge contract using `truffle migrate`.

Remaining minted tokens are burnt by sending them to the zero address.

To start the chain, nothing more than `./childchain.sh` has to be executed. The validators can be configured using `config.json` as well as their initial stakes.
