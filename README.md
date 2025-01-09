# eth-stateless

A toolbox for various Ethereum stateless tasks, such as:

- Merkle Patricia Trie preimages exporter and verifier.

## Prerequisites

- Rust toolchain (stable)
- Cargo
- `--datadir` folder of a synced full-node Reth (i.e: archive node _not_ required)

## Run

```text
Usage: preimages --datadir <DATADIR> <COMMAND>

Commands:
  generate  Generate preimage file
  verify    Verify preimage file
  help      Print this message or the help of the given subcommand(s)

Options:
  -d, --datadir <DATADIR>  Reth datadir path
  -h, --help               Print help
```

### Preimages

The tool provides two subcommands for preimages:

- `generate`: Generate preimage file
- `verify`: Verify preimage file

Two ordering modes are supported:

- `--plain`: Use plain ordering
- `--eip7748`: Use EIP-7748 ordering (i.e: hashed)

### Generate

```text
$ cargo run -p preimages -- generate --help
Generate preimage file

Usage: preimages --datadir <DATADIR> generate [OPTIONS] <--plain|--eip7748>

Options:
      --output-path <PATH>  Preimages file output path [default: preimages.bin]
      --plain               Use plain ordering
      --eip7748             Use EIP-7748 ordering (i.e: hashed)
  -h, --help                Print help
```

Examples:

```text
$ cargo run -p preimages --release -- --datadir=<reth datadir path> generate --plain
[1/1] Generating preimage file...
#####>-------------------------------------------- 10% [eta: 32m] 0x19eaf81a0c1215b7e50524f42594d9496e0ec640
```

```text
$ cargo run -p preimages --release -- --datadir=<reth datadir path> generate --eip7748
[1/2] Ordering account addresses by hash...
#################################################> 100% [eta: 0s] fffec5f54c839fc4a744bebaede23b6e4904007c                                                                                                                
[2/2] Generating preimage file...
#####>-------------------------------------------- 11% [eta: 49m] 1cb3c5ece6021f2d9bf63ba877f8dfc717db509ed66431bebb90c60fedb551ba
```

### Verify

```text
Verify preimage file

Usage: preimages --datadir <DATADIR> verify [OPTIONS] <--plain|--eip7748>

Options:
  -i, --preimages-file-path <PATH>  Preimages file path [default: preimages.bin]
      --plain                       Use plain ordering
      --eip7748                     Use EIP-7748 ordering (i.e: hashed)
  -h, --help                        Print help
```

Example verifying a generated `--eip7748` preimage file:

```text
$ cargo run -p preimages --release -- --datadir=/fast/reth/reth_data verify --path preimages.bin --eip7748 
#>------------------------------------------------ 2% [eta: 54m] 063f6a4b1968bd386869d8f9083e6d5b9525ccf980ab4d4d8d42d824dccaf1ab
```

If we try to verify it with `--plain` it should obviously fail since the expected ordering is different:

```text
$ cargo run -p preimages --release -- --datadir=/fast/reth/reth_data verify --path preimages.bin --plain  
[1/2] Verifying provided preimage file...
Error: Address 0xEA46927B4Fc92248d052299FBFCC6778421930C6 preimage mismatch
```

## LICENSE

MIT.
