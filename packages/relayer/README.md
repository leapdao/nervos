# Bridge

Functionality to listen for events and relay them across chains

### Redis

We use redis two track two things
- Keep count of last processed block
- QueueRunner that relays event data across the listeners
- We have two queues on ckb side using bridgeHash and evm side using bridge address

