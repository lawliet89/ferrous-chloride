# fuzz

`Makefile` commands have to be run in this directory.

## Setup

```bash
cargo install --force honggfuzz
```

You might need some development headers for honggfuzz

```bash
sudo apt-get install binutils-dev libunwind-dev
```

## Fuzz

```bash
HFUZZ_RUN_ARGS="--input fixtures" cargo hfuzz run fuzz  # or `make fuzz`
```

`./fixtures/` is a symlink to the fixtures directory in `/lib`.

## Debug

```bash
cargo hfuzz run-debug fuzz hfuzz_workspace/fuzz/*.fuzz # or `make debug`
```

Crashes will be saved in `./hfuzz_workspace/fuzz/*.fuzz`. You will need `lldb` for debugging.

```bash
sudo apt-get install lldb
```

You can specify a `.fuzz` file to load by replacing `*.fuzz` in the command.

## Clean

You can remove `hfuzz_workspace` and `hfuzz_target`, or run `make clean`.
