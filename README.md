# FinalBiome Impex

A utility to easily create a game spec for FinalBiome testnet.

The utility allows you to download the full specification of the game from FinalBiome and load it back.
This is useful when testing game mechanics on test nodes.

To download the specification, only the address of the game is needed.

Loading a spec requires an organization seed and a manager seed, on behalf of which the game is auto-configured based on the spec.

For convenience, the specification is unloaded in json, which allows it to be stored in version control systems and control changes.

Note:
When creating a game from a specification, all the managers that were in the original game are created. The manager under whose name the game is created is also added to the list of managers (if it is not already there)

## Examples

**Export**
```
finalbiome-impex export -g ./game_spec.json -o 5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw
```
**Import**
```
finalbiome-impex import -g ./game_spec.json -s 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a -m 0x398f0c28f98885e046333d4a41c19cee4c37368a9832c6502f6cfd182e2aef89
```

or with phrase

```s
cargo run import -g ./game_spec.json -s //Alice -m //Bob
```

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
