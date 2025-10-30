# 🪙 Blocksmith — Minimal Rust Blockchain Node

**Blocksmith** is a lightweight, educational blockchain node written in **Rust**, designed to demonstrate:
 
- Basic blockchain implementation;
- Proof-of-Work (PoW) mining;
- Peer-to-Peer (P2P) networking using **libp2p**;
- An async **API service** for transaction submission.

------------------------------------------------------------------------------------------------------------------------

## ⚙️ Architecture Overview

Each node consists of several asynchronous modules running concurrently.

|      Module       | Description                                                                                                            |
|:-----------------:|:-----------------------------------------------------------------------------------------------------------------------|
|    **Mining**     | Computes PoW for the next block in a background thread. Automatically stops when a valid block is received from peers. |
|   **P2P Node**    | Handles peer discovery, connection establishment/closing, message propagation, and block/transaction broadcast.        |
|  **API Service**  | Provides HTTP endpoint to submit transactions.                                                                         |
| **Input Service** | Provides ability to query the blockchain state using CLI.                                                              |
|   **App State**   | Shared mutable state protected by `Arc<Mutex<_>>` for safe concurrent access.                                          |

All modules coordinate through **Tokio channels** and a shared `AppState` structure.

------------------------------------------------------------------------------------------------------------------------

## 🚀 Features

✅ Proof-of-Work (PoW) mining  
✅ Libp2p peer discovery and gossiping  
✅ Transaction and block propagation  
✅ Modular async design (`Tokio` tasks + blocking miners)  
✅ JSON and efficient bincode serialization  

------------------------------------------------------------------------------------------------------------------------

## 🧠 How Mining Works

Each node mines new blocks using simplified PoW algorithm which generates approximately 100000 hashes per second. When
another node broadcasts a valid block, the current miner stops mining its own block and moves on to the next one. The
same happens when the current miner mines a new block - it broadcasts it and all other miners stop with current mining
process and move on to the next block mining.

------------------------------------------------------------------------------------------------------------------------

## 🛠️ Running Locally

### Prerequisites

- `Rust` (>= 1.8)
- `Cargo`
- `libclang` (for `libp2p`)

### Install dependencies
```shell
cargo build
```

### Run a single node
```shell
cargo run | bunyan
```

Nodes automatically connect via `libp2p::Swarm`.

------------------------------------------------------------------------------------------------------------------------

## ⚙️ Configuration

It is possible to skip mining of the genesis block in all nodes which are launched after the very first one by setting
up `MINE_GENESIS` environmental variable to `false` either in the `.env` file or as global environmental variable.

It is also possible to configure API service host and port by modifying them in the `configuration.yaml` file.

------------------------------------------------------------------------------------------------------------------------

## 🧰 Tech Stack

- `Rust` + `Tokio` — concurrency runtime
- `libp2p` — Peer-to-Peer networking
- `sha2` — hashing
- `serde` + `bincode` — serialization
- `actix-web` — API server
- `tracing` — telemetry and structured logging

------------------------------------------------------------------------------------------------------------------------

## 💬 Future Improvements

✅ Implement changeable mining difficulty algorithm  
✅ Replace Proof-of-Work (PoW) algorithm with Proof-of-Stake (PoS) algorithm  
✅ Add persistent storage (LevelDB or RocksDB)  
✅ Implement peer reputation and bootstrapping for faster peers discovering  
✅ Implement enhanced block/transaction/chain validation  
✅ Add WebSocket real-time event stream
