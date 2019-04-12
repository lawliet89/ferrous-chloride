build:
	RUSTFLAGS="-Clink-arg=-fuse-ld=gold" cargo afl build

fuzz:
	AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 AFL_SKIP_CPUFREQ=1 cargo afl fuzz -i fixtures-symlink -o out ./target-symlink/debug/fuzz-afl

install:
	cargo install --force afl
