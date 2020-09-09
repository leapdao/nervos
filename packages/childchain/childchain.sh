#!/bin/bash
KEY="mykey"
CHAINID=8
MONIKER="localtestnet"
VALIDATORS=("0xf3beac30c498d9e26865f34fcaa57dbb935b0d74" "0xe10f3d125e5f4c753a6456fc37123cf17c6900f2" "0xc3ccb3902a164b83663947aff0284c6624f3fbf2" "0x82e8c6cf42c8d1ff9594b17a3f50e94a12cc860f")
VALUE=
# remove existing daemon and client
rm -rf ~/.ethermint*

make install

ethermintcli config keyring-backend test > /dev/null

# Set up config for CLI
ethermintcli config chain-id $CHAINID
ethermintcli config output json
ethermintcli config indent true
ethermintcli config trust-node true

# if $KEY exists it should be deleted
ethermintcli keys add $KEY --algo "eth_secp256k1"
pk=$(ethermintcli keys unsafe-export-eth-key $KEY)

# get version
ethermintd version

# Set moniker and chain-id for Ethermint (Moniker can be anything, chain-id must be an integer)
ethermintd init $MONIKER --chain-id $CHAINID

# Allocate genesis accounts (cosmos formatted addresses)
ethermintd add-genesis-account $(ethermintcli keys show $KEY -a) 10100000000000000000000000000aphoton,1000000000000000000stake

# Sign genesis transaction
ethermintd gentx --name $KEY --keyring-backend test

# Collect genesis tx
ethermintd collect-gentxs

# Run this to ensure everything worked and that the genesis file is setup correctly
ethermintd validate-genesis

# Command to run the rest server in a different terminal/window
echo -e '\nrun the following command in a different terminal/window to run the REST server and JSON-RPC:'
echo -e "ethermintcli rest-server --laddr \"tcp://localhost:8545\" --unlock-key $KEY --chain-id $CHAINID --trace\n"

# Start the node (remove the --pruning=nothing flag if historical queries are not needed)
ethermintd start --pruning=nothing --rpc.unsafe --log_level "main:info,state:info,mempool:info" --trace > /dev/null &

sleep 2

coproc ethermintcli rest-server --laddr "tcp://localhost:8545" --unlock-key $KEY --chain-id $CHAINID --trace > /dev/null

sleep 2

address=$(truffle migrate --network development | perl -lne 'print "$1" if /(?p)deployed-address:(.*)/' | sed -n '1p')
node transactions --experimental-top-level-await $pk $address
