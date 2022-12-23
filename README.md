# FinalBiome Impex

A utility to easily create a game spec for FinalBiome testnet.

## Usage
**Commands**:  
  `export`  Export game spec to the file  
  `import`  Create game from game spec file  
  `help`    Print this message or the help of the given subcommand(s)

**Options**:  
  `-h`, `--help`     Print help information  
  `-V`, `--version`  Print version information  

### Export

```sh
finalbiome-impex export [OPTIONS] --organization <ORGANIZATION>
```

**Options**:
```
  -e, --endpoint <ENDPOINT>          RPC endpoint of the network node [default: ws://127.0.0.1:9944]
  -o, --organization <ORGANIZATION>  Game address in SS58 format
  -g, --game-spec <GAME_SPEC>        Path to the game file to which the game configuration will be written [default: ./game_spec.json]
  -w, --overwrite                    Whether to overwrite the file if it exists?
  -h, --help                         Print help information
```

### Import

```sh
finalbiome-impex import [OPTIONS] --game-spec <GAME_SPEC> --organization-seed <ORGANIZATION_SEED> --manager-seed <MANAGER_SEED>
```

**Options**:
```
  -e, --endpoint <ENDPOINT>
          RPC endpoint of the network node [default: ws://127.0.0.1:9944]
  -g, --game-spec <GAME_SPEC>
          Path to the game file from which the game configuration will be read
  -s, --organization-seed <ORGANIZATION_SEED>
          Game organization account key seed. May be a secret seed or secret URI.
  -m, --manager-seed <MANAGER_SEED>
          Game manager on whose behalf it will be configured. May be a secret seed or secret URI.
  -h, --help
          Print help information
```


## Development
Use the subxt-cli tool to download the metadata for FinalBiome target runtime from a node.

1. Install:
```sh
cargo install subxt-cli
```
Save the encoded metadata to a file:
```sh
subxt metadata -f bytes > ./artifacts/finalbiome_metadata.scale
```
