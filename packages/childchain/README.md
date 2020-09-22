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

After the Ethermint chain is started, a genesis deployment account is used to deploy the Bridge contract using `truffle migrate`.

To start the chain, nothing more than `./childchain.sh` has to be executed.
At present a single validator has been loaded using a mnemonic key, in the future, configuration may be changed through config.json.

# setup for a production version

For any validator setup the following components shall be needed.
Namely a deployment server to communicate the operators(validators) to the parent chain, read staking information from the parent chain, distributing template genesis files, collecting genesis transactions as well as redistributing those to the operators.Finally, the childchain bridge shall be deployed by the deployment server after the chain has started.

The simplest setup for a production version will be a single validator setup, more commonly a multi validator setup is expected to be used.

# single validator setup

![single validator setup](https://i.imgur.com/KpFOahO.jpg)

# multi validator setup

![multi validator setup](https://i.imgur.com/JUbWrCD.jpg)
