# Running the node

To start the node make sure to have ethermint (v0.1.0 or higher) installed and have your GO Path exposed:

```
git clone https://github.com/ChainSafe/ethermint
cd ethermint
make install
```

# Run the tests

To compile the contracts run `truffle build`, to run the tests run `truffle test`.

# Prerequisites

Requirements to run this script are the installation of Ethermint, truffle, node.js, perl and sed.

# Start-up script

The underlying bash-Script `childchain.sh` starts the Ethermint chain, initializes a genesis account as well as starting a RPC server. To stop the chain the command `pkill ethermint` should be used or analogous command.

After the Ethermint chain is started, a genesis deployment account is used to deploy the Bridge contract using `truffle migrate`.

To start the chain, nothing more than `./childchain.sh` has to be executed, at present this runs a local development chain with a single validator and a bridge deployment.
At present a single validator has been loaded using a mnemonic key, in the future, configuration may be changed through config.json.
Validators will sign genesis transactions on their own machine, therefore limiting exposure to their private key to their own device. The usage of hardware security modules is recommended. The deployment server creates a new account to deploy the childchain bridge from which will be emptied afterwards.

Log files will be created for the RPC server, the chain and the deployment account creation.

On successful deployment blocks will be produced and the JSON RPC server accessible on port 8545.

# setup for a production version

For any validator setup the following components shall be needed.
For production, firstly the parent chain bridge shall be deployed with stakes for the childchain validators already submitted.
Namely a deployment server to communicate the operators(validators) to the parent chain, read staking information from the parent chain, distributing template genesis files, collecting genesis transactions as well as redistributing those to the operators.
The communication with the deployment server will likely require manual intervention by the operators.
The operators have to communicate their IP addresses to each other through the deployment server for P2P communication and block production to start.
Finally, the childchain bridge shall be deployed by the deployment server after the chain has started, the maximum amount of tokens an EVM chain can store (16^30) minus the validator stakes will be sent to the childchain bridge. In this way we will be able to represent the growing amount of tokens on the Nervos parent chain.

The simplest setup for a production version will be a single validator setup, more commonly a multi validator setup is expected to be used.

# single validator setup

![single validator setup](https://i.imgur.com/KpFOahO.jpg)

# multi validator setup

![multi validator setup](https://i.imgur.com/JUbWrCD.jpg)
