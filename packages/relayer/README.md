# Bridge

Functionality to listen for events and relay them across chains

The flow is listen and relay. 
- We make use of redis queues to do the relay across chains
- All relays are queued then processed by the handle function in either ckb.ts or evm.ts

### Lock Flow
- User deposits to bridge on nervos network
- Listen function on ckb.ts filters out the deposit transaction
- `_relayLock` function formats transaction and adds to queue
- Handle function on evm.ts reads the queue and retrieves transaction
- `_collectLock` function sends transaction to childchain with validator signature

### Unlock 
- User sends amount to childchain bridge address
- Listen function on evm.ts retrieves events and we check for Burn event
- `_collectUnLock` function sends transaction to childchain with validator signature
- Listen function on evm.ts retrieves events and we check for BurnQuorom event
- `_relayUnLock` function adds event data to queue
- Handle function on ckb.ts reads the queue and retrieves transaction
- `_processUnLockRelay` creates a nervos transactions sends transaction to nervos network


## Launch Service
To start the service on console run `npm run start`

*First time run `npm run deploy` copy address values to config file then `npm run start`*

## Config

```json
{
    "bridgeAddress":"0x49D7858Cb3b598d79de772B0821af8a67e424c0c",
    "bridgeHash": "BridgeScriptHash",
    "redis": {
        "host": "localhost",
        "port": 6379
    }
}
```

We use a config.json file to store bridge address (evm bridge) and bridge hash (nervos bridge).
Config file will be built through start up script in future.

To get the bridge address run `npm run deploy` 
Copy the address to the config file. 

*TODO get bridge hash and save to config file*

## Redis v6.0.8
Environment should have redis running in the background on port 6379.

We use redis to track two things
- Keep count of last processed block
- QueueRunner that relays event data across the listeners
- We have two queues on ckb side using bridgeHash and evm side using bridge address
