# FastShell

A small shell interpreter, which translates shell commands into PL/pgSQL and thus enables full pipeline optimization.

If used with a normal file system, this interpreter only supports branch-free scripts. This limitation arises from limitations within Postgres, which prohibit the use of the `copy` command within PL/pgSQL.

In conjunction with SQLFS, `copy` is not needed to load the data, since the data is already present in the database. However, shell commands which are not natively supported will not be functional, since the fall back solution also requires the `copy` command to pipe the data into an external shell command.

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

## Supported Commands

* `cat <file>`
* `head -n <line count>`
* `tail -n <line count>`
* `sort <flags>`
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

## Examples

see test cases in `executor.rs`