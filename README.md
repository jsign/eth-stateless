# eth-stateless

A toolbox for various Ethereum stateless tasks, such as:

- Merkle Patricia Trie preimages exporter, verifier and frequency analysis.

## Prerequisites

- Rust toolchain (stable)
- Cargo
- `--datadir` folder of a synced full-node Reth (i.e: archive node _not_ required)

## Preimages

```text
Usage: preimages --datadir <DATADIR> <COMMAND>

Commands:
  generate           Generate preimage file
  verify             Verify preimage file
  storage-slot-freq  Analyze storage-slot 29-byte prefix frequency and size impact
  help               Print this message or the help of the given subcommand(s)

Options:
  -d, --datadir <DATADIR>  Reth datadir path
  -h, --help               Print help
```

### Commands

The tool provides two subcommands for preimages:

- `generate`: Generate preimage file
- `verify`: Verify preimage file
- `storage-slot-freq` does a frequency analysis of the 29-byte prefix of storage slots

For the `generate` and `verify` commands, two ordering modes are supported:

- `--plain`: Use plain ordering (i.e. unhashed)
- `--eip7748`: Use EIP-7748 ordering (i.e. trie(s) DFS (hashed))

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
Database block number: 21547467
[1/1] Generating preimage file...
#####>-------------------------------------------- 10% [eta: 32m] 0x19eaf81a0c1215b7e50524f42594d9496e0ec640
```

```text
$ cargo run -p preimages --release -- --datadir=<reth datadir path> generate --eip7748
Database block number: 21547467
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
Database block number: 21547467
#>------------------------------------------------ 2% [eta: 54m] 063f6a4b1968bd386869d8f9083e6d5b9525ccf980ab4d4d8d42d824dccaf1ab
```

If we try to verify it with `--plain` it should obviously fail since the expected ordering is different:

```text
$ cargo run -p preimages --release -- --datadir=/fast/reth/reth_data verify --path preimages.bin --plain  
Database block number: 21547467
[1/2] Verifying provided preimage file...
Error: Address 0xEA46927B4Fc92248d052299FBFCC6778421930C6 preimage mismatch
```

### Storage slots 29-byte prefix frequency and size impact analysis

```text
$ cargo run -p preimages --release -- --datadir=/fast/reth/reth_data storage-slot-freq --help
Analyze top N storage slot frequency

Usage: preimages --datadir <DATADIR> storage-slot-freq

Options:
  -h, --help       Print help
```

Example:

```text
$ cargo run -p preimages --release -- --datadir=/fast/reth/reth_data storage-slot-freq
Database block number: 21547467
#################################################> 100% [eta: 0s] fffffffff15abf397da76f1dcc1a1604f45126db                                                                                                                           
Top 25 storage slot 29-byte prefix repetitions:
0000000000000000000000000000000000000000000000000000000000: 56944638 (4.65%) ~1574MiB (cumm 1574MiB)
f3f7a9fe364faab93b216da50a3214154f22a0a2b415b23a84c8169e8b: 13665589 (1.12%) ~377MiB (cumm 1952MiB)
8a35acfbc15ff81a39ae7d344fd709f28e8600b4aa8c65c6b64bfe7fe3: 9425916 (0.77%) ~260MiB (cumm 2213MiB)
f652222313e28459528d920b65115c16c04f3efc82aaedc97be59f3f37: 8546483 (0.70%) ~236MiB (cumm 2449MiB)
405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3: 7701011 (0.63%) ~212MiB (cumm 2662MiB)
a66cc928b5edb82af9bd49922954155ab7b0942694bea4ce44661d9a87: 3509056 (0.29%) ~97MiB (cumm 2759MiB)
...
```

## LICENSE

MIT.
