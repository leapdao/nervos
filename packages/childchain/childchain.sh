#!/bin/bash
KEY="mykey"
CHAINID=8
MONIKER="localtestnet"
ALICE_PK="0x278a5de700e29faae8e40e366ec5012b5ec63d36ec77e8a2417154cc1d25383f"
# remove existing daemon and client
rm -rf ~/.ethermint*

make install

ethermintcli config keyring-backend test > key_creation.log

# Set up config for CLI
ethermintcli config chain-id $CHAINID
ethermintcli config output json
ethermintcli config indent true
ethermintcli config trust-node true

# if $KEY exists it should be deleted
ethermintcli keys add $KEY --algo "eth_secp256k1"
pk=$(ethermintcli keys unsafe-export-eth-key $KEY)

# creates validator key and adds it to keyring
ethermintcli keys delete validator || true
echo "slide illness naive return canvas almost seven eager custom runway fish panther gas choice wall moral fork fine muffin report sword acid decorate steel"| ethermintcli keys add validator --recover

# get version
ethermintd version

# Set moniker and chain-id for Ethermint (Moniker can be anything, chain-id must be an integer)
ethermintd init $MONIKER --chain-id $CHAINID

# Allocate genesis accounts (cosmos formatted addresses) 16^30 for deployment account
ethermintd add-genesis-account $(ethermintcli keys show $KEY -a) 1329227995784915862903807060280344576aphoton
ethermintd add-genesis-account $(ethermintcli keys show "validator" -a) 10000000000000000000aphoton,5000000000000000000stake

# Sign genesis transaction
ethermintd gentx --name "validator" --keyring-backend test

# Collect genesis tx
ethermintd collect-gentxs

# Run this to ensure everything worked and that the genesis file is setup correctly
ethermintd validate-genesis

# Command to run the rest server in a different terminal/window
echo -e '\nrun the following command in a different terminal/window to run the REST server and JSON-RPC:'
echo -e "ethermintcli rest-server --laddr \"tcp://localhost:8545\" --unlock-key $KEY --chain-id $CHAINID --trace\n"

# Start the node (remove the --pruning=nothing flag if historical queries are not needed)
ethermintd start --pruning=nothing --rpc.unsafe --log_level "main:info,state:info,mempool:info" --trace > ethermint-chain.log &

#wait for chain to initialize before starting RPC server
sleep 2

#starting RPC server
coproc ethermintcli rest-server --laddr "tcp://localhost:8545" --unlock-key $KEY --chain-id $CHAINID --trace > ethermint-rpc.log

#waiting for RPC server startup before migrating the Bridge contract
sleep 2

address=$(truffle migrate --network development | perl -lne 'print "$1" if /(?p)deployed-address:(.*)/' | sed -n '1p')
node transactions --experimental-top-level-await $pk $address
