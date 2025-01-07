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
##############>----------------------------------- 28% [eta: 9m] 0x47ece2c052834097b1e65044dc096c034369c2d4
```

## LICENSE

MIT.
