# eth-stateless

A toolbox for various Ethereum stateless tasks, such as:

- Merkle Patricia Trie preimages exporter.

## Project Structure

- `preimages/` - Preimage collection and analysis

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

Usage: preimages --datadir <DATADIR> generate [OPTIONS]

Options:
  -o, --output-path <PATH>  Preimages file output path [default: preimages.bin]
  -h, --help                Print help
```

Example:

```text
$ cargo run -p preimages --release -- --datadir=<reth datadir path> generate
#####>-------------------------------------------- 10% [eta: 32m] 0x19eaf81a0c1215b7e50524f42594d9496e0ec640
```

## LICENSE

MIT.
