fuzz:
	HFUZZ_RUN_ARGS="--input fixtures-symlink" cargo hfuzz run fuzz

clean:
	cargo hfuzz clean
	rm -rf ./hfuzz_workspace

debug:
	cargo hfuzz run-debug fuzz hfuzz_workspace/fuzz/*.fuzz

install:
	apt-get install binutils-dev libunwind-dev lldb
	cargo install --force honggfuzz
