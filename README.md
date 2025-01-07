# eth-stateless

A toolbox for various Ethereum stateless tasks, such as:

- Merkle Patricia Trie preimages exporter with plain or EIP-4762 output ordering.

## Prerequisites

- Rust toolchain (stable)
- Cargo
- `--datadir` folder of a synced full-node Reth (i.e: archive node _not_ required)

## Run

```text
Usage: preimages --datadir <DATADIR> <COMMAND>

Commands:
  generate  Generate preimage file
  help      Print this message or the help of the given subcommand(s)

Options:
  -d, --datadir <DATADIR>  Reth datadir path
  -h, --help               Print help
```

### Preimages exporting

```text
$ cargo run -p preimages -- generate --help
Generate preimage file

Usage: preimages --datadir <DATADIR> generate [OPTIONS] <--plain|--eip4762>

Options:
      --output-path <PATH>  Preimages file output path [default: preimages.bin]
      --plain               Use plain ordering
      --eip4762             Use EIP-4762 ordering (i.e: hashed)
  -h, --help                Print help
```

Example with `--plain` ordering:

```text
$ cargo run -p preimages --release -- --datadir=<reth datadir path> generate --plain
[1/1] Generating preimage file...
#####>-------------------------------------------- 10% [eta: 32m] 0x19eaf81a0c1215b7e50524f42594d9496e0ec640
```

Example with `--eip4762` ordering:

```text
$ cargo run -p preimages --release -- --datadir=<reth datadir path> generate --eip4762
[1/2] Ordering account addresses by hash...
#################################################> 100% [eta: 0s] fffec5f54c839fc4a744bebaede23b6e4904007c                                                                                                                
[2/2] Generating preimage file...
#####>-------------------------------------------- 11% [eta: 49m] 1cb3c5ece6021f2d9bf63ba877f8dfc717db509ed66431bebb90c60fedb551ba
```

## LICENSE

MIT.
