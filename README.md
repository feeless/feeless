# ⋰·⋰ Feeless

[![Crates.io](https://img.shields.io/crates/v/feeless?style=flat-square)](https://crates.io/crates/feeless)
[![docs.rs](https://img.shields.io/docsrs/feeless?style=flat-square)](https://docs.rs/feeless/)
[![GitHub last commit](https://img.shields.io/github/last-commit/feeless/feeless?style=flat-square)](https://github.com/feeless/feeless/graphs/commit-activity)
[![MIT OR Apache-2.0 Licence](https://img.shields.io/crates/l/feeless?style=flat-square)](https://github.com/dtolnay/rust-faq#why-a-dual-mitasl2-license)

## What is Feeless?

**Feeless** is a **Nano** cryptocurrency node, wallet, tools, and Rust crate. This is not the official project for Nano,
only an implementation written in Rust. The official Nano node
implementation [lives here](https://github.com/nanocurrency/nano-node).

🚸 This is a work in progress. The API will probably change without notice until `v0.2`. 🚸

I decided to start this project as a personal adventure of understanding Nano. I give no promises about my future
motivation to complete this project 🤐.

## Documentation

Please visit the documentation website for general information, features, installation, CLI usage and more.

https://feeless.dev/

## What is Nano?

**Nano** is digital money that significantly improves on **Bitcoin** and other cryptocurrencies.

The main features of **Nano** are:

* No transaction fees.
* Extremely fast to send money—less than 1 second for 100% confirmation.

  <sup>
    Bitcoin takes 10 minutes on average for ~80%<sup>1</sup> confirmation.
    Nano is more asynchronous than Bitcoin—individual transactions are voted on separately from the rest of the network.
  </sup>
* Highly decentralized.

  <sup>Using the Nakamoto coefficient measurement, it is more decentralized than Bitcoin<sup>2 3</sup>.
* No inflation.
* Green—Massively less energy use than Bitcoin.

For more information on what Nano is, see the Nano documentation: https://docs.nano.org/what-is-nano/overview/

Nano is also known as: Nano cryptocurrency, Nano coin, RaiBlocks.

<sup>
1. The Bitcoin white paper, under section 11 "Calculations" explains there's a ~80% chance for an attacker with 10% mining power to overtake the longest chain. https://bitcoin.org/bitcoin.pdf
2. Measuring Decentralization in Bitcoin and Ethereum using Multiple Metrics and Granularities https://arxiv.org/pdf/2101.10699.pdf
3. List of representative nodes showing a Nakamoto coefficient of 8 at the time of writing (2021-02) https://nanocharts.info/

</sup>

## Credits

Please see https://feeless.dev/docs/overview/credits

## License

Licensed under either of these at your option:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
