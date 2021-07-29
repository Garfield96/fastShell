# FastShell

A small shell interpreter, which translates shell commands into SQL and thus enables full pipeline optimization.

## Build Docker Image
```sh
docker build -t fastshell docker/
```
## Run Docker
```sh
docker run --rm -it -v $(pwd):/fastshell fastshell
```

## Build

```sh
cargo build --release
```

## Usage

Execute command
```sh
target/release/fastShell -c "cat ..."
```

Execute script
```sh
target/release/fastShell <script>
```

`&&` and `||` are treated like `;`.

Only write redirects `>` are supported.

### Postgres

Adapt username and password and make sure a DB called `shell` exists.

## Supported Commands

* `cat <file>`
* `head -n <line count>`
* `tail -n <line count>`
* `sort`
  * `-r`,`--reverse`
  * `-b`,`--ignore-leading-blanks`
  * `-f`,`--ignore-case`
* `grep <pattern>`
* `uniq <flag> lines`
  * `-c`,`--count`
  * `-u`, `--unique`
* `wc`
  * `-l`,`--lines`
  * `-c`, `--chars`
  
All commands support `--help`. 