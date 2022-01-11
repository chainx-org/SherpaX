#[sherpax-cli](https://github.com/chainx-org/sherpax-cli.git)

## Check Balance

- install
```bash
$ git clone https://github.com/chainx-org/sherpax-cli.git
$ cd sherpax-cli
$ cargo build --release
```
- run
```bash
$ ./target/release/sherpax-cli check-balance --help
sherpax-cli-check-balance 0.1.0
Arguments required for creating and sending an extrinsic to a sherpax node

USAGE:
    sherpax-cli check-balance [FLAGS] [OPTIONS]

FLAGS:
    -h, --help             Prints help information
        --print-details    Enable print the detail info(every 10000 blocks)
    -V, --version          Prints version information

OPTIONS:
        --block-number <block-number>    The specified block number
        --url <url>                      Websockets url of a sherpax node [default: ws://localhost:9977]

```

```bash
$ ./target/release/sherpax-cli check-balance --url ws://127.0.0.1:9977
{
    "block_number":46,
    "reserved":"0",
    "transferable_exclude_treasury":"3629783363038086000000000",
    "treasury_balance":"9322852245775200000000000",
    "vesting_locking":"8047364391186714000000000",
    "vote_locking":"0"
}
```
