# FastShell

A small shell interpreter, which translates shell commands into SQL and thus enables full pipeline optimization.

## Build

```sh
cargo build --release
```

## Usage

```sh
target/release/fastShell "cat ..."
```

`&&` and `||` are treated like `;`.

Only write redirects `>` are supported.

### Postgres

Adapt username and password and make sure a DB called `shell` exists.

## Supported Commands

* `cat`
* `head <N>`
* `tail <N>`
* `sort`
  * `desc`
* `grep lines <pattern>`
* `uniq <flag> lines`
  * `-c`,`--count`
  * `-u`, `--unique`
* `wc`
  * `-l`,`--lines`
  * `-c`, `--chars`