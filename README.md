# feeless - A Nano Cryptocurrency node, wallet, tools, and Rust library.

**⚠ This is a work in progress. It's not ready to use as a real node! ⚠**

As a personal adventure into understanding Nano, I decided to write a Nano node in Rust. The original implementation is
in C++. I give no promises about my future motivation to complete this project 🤐.

## What is Nano?

**Nano** is a decentralized money that is significantly improves on Bitcoin and other cryptocurrencies.

The main features of **Nano** are:

* No transaction fees.
* Extremely fast to send @ less than 1 second. (Bitcoin takes at least 10 minutes.)
* Decentralized (as much as Bitcoin).
* No inflation.
* Significantly less energy use than Bitcoin.

Other names: Nano cryptocurrency, Nano coin, Rai Blocks.

For more information on what Nano is, see the Nano documentation: https://docs.nano.org/what-is-nano/overview/

## Goals

### Rust crate

* A complete library that a Rust developer can use to manage wallets, keys, blocks.
* Compatibility with the JSON RPC server.

### Tooling

* A command line tool for particular actions, e.g. generating seeds, conversions between keys, addresses, etc.
* An RPC command line interface for the JSON RPC server.

### Nano node

* A functional node with as much business logic as I can understand from the C++ implementation.
* It has to perform well enough to help the network. I don't want Nano to slow down if people start using this! 🤦‍♀️
* Correct Rebroadcasting rules
* Representative voting
* Bootstrapping

## Non-goals

* Only support protocol version 18+
* No UDP support
* No user interface

## Task list

A medium term task list:

- [x] Seeds
  - [x] Mnemonic (word list) seed generation/parsing (BIP39)
  - [x] Derive keys from mnemonic (BIP33)
  - [x] Hex seeds
- [x] Keys (ed25519/blake2b)
  - [x] Private keys
  - [x] Public keys
  - [x] Nano addresses
    - [x] Validation
    - [x] Parsing
    - [x] Conversion to/from public keys
- [x] Nano amount conversions
  - [x] raw
  - [x] nano
  - [x] Mnano/NANO
- [ ] Proof of work
- [ ] Blocks
  - [x] Hashing
  - [ ] Work
  - [ ] State blocks
  - [ ] <v18 blocks?
- [ ] Packet dissector
  - [x] Parse hex dump from Wireshark
  - [ ] Parse pcap
  - [x] Dump some message types to console
- [ ] Node
  - [ ] Configuration
    - [x] Initial command line interface
    - [ ] Network
    - [ ] Database
    - [ ] ...
  - [ ] Networks
    - [x] Live (Don't worry, I'm only connecting to my own node at the moment!)
    - [ ] Test
    - [ ] Beta
  - [ ] Bootstrap peer connection (peering.nano.org)
  - [x] Validate given peer network
  - [ ] Validate given peer versions
  - [ ] Multiple peer connectivity (currently only connects to one peer)
    - [ ] Configurable maximum peer limit
  - [x] Header parsing
    - [x] Network
    - [x] Versions
    - [x] Extensions
      - [x] Handshake query/response flags
      - [x] Count
      - [x] Block type
      - [ ] Telemetry size
      - [ ] Extended params present
  - [ ] Logic
    - [ ] Rebroadcasting
    - [ ] Representatives
    - [ ] ...
  - [ ] Messages
    - [ ] Node ID Handshake
      - [x] Serialize (TODO: needs small refactor)
      - [x] Deserialize
      - [x] Send cookie
      - [ ] Cookie/peer store and logic
      - [x] Validate response
    - [ ] Confirm Req
      - [ ] Serialize
      - [ ] Deserialize
        - [x] Hash pairs
        - [ ] Block selector
      - [ ] Handle response
    - [ ] Confirm Ack
      - [ ] Serialize
      - [ ] Deserialize
        - [x] Vote by hash
        - [ ] Block
    - [ ] Keepalive
      - [ ] Serialize
      - [x] Deserialize
    - [ ] Publish
      - [ ] Serialize
      - [x] Deserialize
        - [x] State blocks
        - [ ] Other blocks
    - [ ] Bulk pull
    - [ ] Bulk pull account
    - [ ] Bulk pull blocks
    - [ ] Bulk push
    - [ ] Telemetry Req
      - [ ] Serialize
      - [x] Deserialize
      - [ ] Collect telemetry
      - [ ] Handle response
    - [ ] Telemetry Ack
    - [ ] Frontier Req
  - [ ] Storage
    - [x] Basic KV store to file
    - [x] Basic cookie/peer storage
    - [ ] Peers
    - [ ] Blocks
    - [ ] ...
  - [ ] RPC
- [ ] Rust
  - [ ] Ask around for a code review
  - [ ] Use either `zerocopy` or make all core types zero-copy with storing `[u8]` and methods as accessors. `zerocopy`
    did work for most things when I tried but had problems with enums. Might revisit.
  - [ ] Use `thiserror` instead of `anyhow`
  - [ ] Github actions CI (including `cargo clippy`)
- [ ] Future things
  - [ ] Performance
    - [ ] Automated comparison
  - [ ] Proof of work
    - [ ] Server
    - [ ] GPU
  - [ ] WASM

## Credits and references

* Thanks to the hard work from the Nano Foundation.
* https://forum.nano.org/, https://old.reddit.com/r/nanocurrency/, Nano Discord: https://chat.nano.org/
  * A very friendly community helping out others and myself.
* https://docs.nano.org/
  * General useful information.
* https://nanoo.tools/
  * Helped me understand technical details on state blocks and hashing, and also with validating conversions between
    things.
* https://github.com/nanocurrency/protocol/blob/master/reference
  * Node protocol specification.
* https://github.com/nanocurrency/nano-node
  * The actual Nano implementation as a source of truth.

## Licence

This project is licenced under both MIT and Apache 2.0.
