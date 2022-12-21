# FinalBiome Impex

A utility to easily create a game spec for testnet.

# Development
Use the subxt-cli tool to download the metadata for FinalBiome target runtime from a node.

1. Install:
```sh
cargo install subxt-cli
```
Save the encoded metadata to a file:
```sh
subxt metadata -f bytes > ./artifacts/finalbiome_metadata.scale
```
