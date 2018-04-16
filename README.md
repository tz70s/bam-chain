# bam-chain

Baby blockchain implementation in rust.

## How to use this?

Install rust and relative tool chain as prerequisition.

```bash
# Open a terminal
cargo build --release
./target/release/bam-chain 8181

# Open another terminal for spawning another miner node.
./target/release/bam-chain 8282

# Open another terminal for sending http request.
# i.e. use httpie in mac.
http POST http://localhost:8181/add_peers < resource/peers_template.json
http GET http://localhost:8181/list_peers

# Post data, a.k.a. mine a new block.
http POST http://localhost:8181/mine Hello=World
# List blocks.
http GET http://localhost:8181/list
# You can see that the added peers get into a same chain(except genesis block).
http GET http://localhost:8282/list
```

Supporting routes:

* **GET** `/` : entry msg.
* **GET** `/list` : list the blockchain in this node.
* **POST** `/mine` : post a data and add a block in the node.
* **POST** `/add_peers` : add peers to this node.
* **GET** `/list_peers` : list peers of this node.

Incoming update: easy deployment, stabilized inter-connection service, introduce PoW/PoS and transactions, wallet UI.

## Acknowledgement

This project is inspired by [naivechain](https://github.com/lhartikk/naivechain).

## License

MIT.