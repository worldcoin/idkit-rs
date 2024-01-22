<a href="https://docs.worldcoin.org/">
  <img src="https://raw.githubusercontent.com/worldcoin/world-id-docs/main/public/images/shared-readme/readme-header.png" alt="" />
</a>

# IDKit (Rust)

[![crates.io](https://img.shields.io/crates/v/idkit.svg)](https://crates.io/crates/idkit)
[![download count badge](https://img.shields.io/crates/d/idkit.svg)](https://crates.io/crates/idkit)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/idkit)

The `idkit` crate provides a simple Rust interface for prompting users for World ID proofs. For our Web and React Native SDKs, check out the [IDKit JS library](https://github.com/worldcoin/idkit-js).

## Usage

```rust
use idkit::{Session, session::{AppId, VerificationLevel, BridgeUrl, Status}};

let session = Session::new(AppId::from_str("app_GBkZ1KlVUdFTjeMXKlVUdFT")?, "vote_1", VerificationLevel::Orb, BridgeUrl::default(), (), None).await?;

// To establish a connection, show a QRCode to the user with the generated URL.
let connect_url = session.connect_url();

loop {
    match session.poll_for_status().await {
        Status::WaitingForConnection | Status::AwaitingConfirmation => {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        },
        Status::Failed(error) => {
            // ...
        },
        Status::Confirmed(proof) => {
            /// ...
        },
    }
}
```

Refer to the [documentation on docs.rs](https://docs.rs/idkit) for detailed usage instructions.

<!-- WORLD-ID-SHARED-README-TAG:START - Do not remove or modify this section directly -->
<!-- The contents of this file are inserted to all World ID repositories to provide general context on World ID. -->

## <img align="left" width="28" height="28" src="https://raw.githubusercontent.com/worldcoin/world-id-docs/main/public/images/shared-readme/readme-world-id.png" alt="" style="margin-right: 0; padding-right: 4px;" /> About World ID

World ID is the privacy-first identity protocol that brings global proof of personhood to the internet. More on World ID in the [announcement blog post](https://worldcoin.org/blog/announcements/introducing-world-id-and-sdk).

World ID lets you seamlessly integrate authentication into your app that verifies accounts belong to real persons through [Sign in with Worldcoin](https://docs.worldcoin.org/id/sign-in). For additional flexibility and cases where you need extreme privacy, [Anonymous Actions](https://docs.worldcoin.org/id/anonymous-actions) lets you verify users in a way that cannot be tracked across verifications.

Follow the [Quick Start](https://docs.worldcoin.org/quick-start) guide for the easiest way to get started.

## 📄 Documentation

All the technical docs for the Wordcoin SDK, World ID Protocol, examples, guides can be found at https://docs.worldcoin.org/

<a href="https://docs.worldcoin.org">
  <p align="center">
    <picture align="center">
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/worldcoin/world-id-docs/main/public/images/shared-readme/visit-documentation-dark.png" height="50px" />
      <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/worldcoin/world-id-docs/main/public/images/shared-readme/visit-documentation-light.png" height="50px" />
      <img />
    </picture>
  </p>
</a>

<!-- WORLD-ID-SHARED-README-TAG:END -->
