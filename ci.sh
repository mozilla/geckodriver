set -ex

# Add provided target to current Rust toolchain if it is not already
# the default or installed.
rustup_target_add() {
	if ! rustup target list | grep -E "$1 \((default|installed)\)"
	then
		rustup target add $1
	fi
}

# Configure rustc target for cross compilation.  Provided with a build
# target, this will determine which linker to use for cross compilation.
cargo_config() {
	local prefix

	case "$TARGET" in
	aarch64-unknown-linux-gnu)
		prefix=aarch64-linux-gnu
		;;
	arm*-unknown-linux-gnueabihf)
		prefix=arm-linux-gnueabihf
		;;
	arm-unknown-linux-gnueabi)
		prefix=arm-linux-gnueabi
		;;
	mipsel-unknown-linux-musl)
		prefix=mipsel-openwrt-linux
		;;
	x86_64-pc-windows-gnu)
		prefix=x86_64-w64-mingw32
		;;
	*)
		return
		;;
	esac

	mkdir -p ~/.cargo
	cat >>~/.cargo/config <<EOF
[target.$TARGET]
linker = "$prefix-gcc"
EOF
}

# Build current crate and print file type information.
cargo_build() {
	cargo build --target $1

	if [[ "$1" =~ "windows" ]]
	then
		file target/$1/debug/geckodriver.exe
	else
		file target/$1/debug/geckodriver
	fi
}

# Run current crate's tests if the current system supports it,
# e.g. the system is not Windows.
cargo_test() {
	# this list is a dump of `rustup target list | grep linux`
	case "$TARGET" in
	aarch64-unknown-linux-gnu)
	arm-linux-androideabi)
	arm-unknown-linux-gnueabi)
	arm-unknown-linux-gnueabihf)
	armv7-unknown-linux-gnueabihf)
	i586-unknown-linux-gnu)
	i686-unknown-linux-gnu)
	i686-unknown-linux-musl)
	mips-unknown-linux-gnu)
	mips-unknown-linux-musl)
	mipsel-unknown-linux-gnu)
	mipsel-unknown-linux-musl)
	powerpc-unknown-linux-gnu)
	powerpc64-unknown-linux-gnu)
	powerpc64le-unknown-linux-gnu)
	x86_64-unknown-linux-gnu)
	x86_64-unknown-linux-musl)
		cargo test --target $1
		;;

	*)
		>&2 echo "not running tests on $1"
		return
		;;
}

main() {
	rustup_target_add $TARGET
	cargo_config $TARGET
	cargo_build $TARGET
	cargo_test $TARGET
}

main
