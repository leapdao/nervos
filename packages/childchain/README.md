# Running the node

To start the node make sure to have ethermint installed and have your GO Path exposed: 

```
git clone https://github.com/ChainSafe/ethermint
cd ethermint
make install
```
To run the chain run `./childchain.sh` and in a second terminal/tab `ethermintcli rest-server --laddr "tcp://localhost:8545" --unlock-key mykey --chain-id 8 --trace`

# Run the tests

To compile the contracts run `truffle build`, to run the tests run `truffle test`.
