PREVDIR="$(pwd)"
cd "$(dirname "$0")"

if [[ "${2:-__NOTHING__}" == "__NOTHING__" ]]; then
	ARCH="x86_64-unknown-linux-musl"
else
	case "$2" in
		"x86" | "x86_64" | "adm64")
			ARCH="x86_64-unknown-linux-musl"
			;;
		"raspberrypi" | "arm-linux-gnueabi" | "arm-linux-gnueabihf" | "arm-unknown-linux-gnueabihf" | "arm-unknown-linux-gnueabi" | "armel" | "armhf" )
			ARCH="arm-unknown-linux-gnueabi"
			export RUSTFLAGS="-C linker=arm-linux-gnueabi-gcc -C target-cpu=arm1176jzf-s"
			;;
		*)
			echo "ERROR: Unknown CPU architecture: $2"
			exit 1
			;;
	esac
fi

shopt -s nullglob

help() {
	echo "test.sh YUNOHOST_SERVER [ARCH]"
	echo "  Start the test suite on remote YUNOHOST_SERVER server."
	echo "  You need SSH key authentication without password."
}

stop_this_shit() {
	cd "${PREVDIR}"
	exit 1
}

if [[ "${1:-__NOTHING__}" == "__NOTHING__" ]]; then
	help
	stop_this_shit
fi

ssh="ssh -q -o PasswordAuthentication=no"

build_check() {
	if ! cargo fmt --check; then
		echo "cargo fmt failed. Exit"
		stop_this_shit
	fi
	if ! cargo build --release --target $ARCH; then
		echo "cargo build failed. Exit"
		stop_this_shit
	fi
	if ! RUSTDOCFLAGS="-D warnings" cargo doc --target $ARCH --no-deps; then
		echo "cargo doc failed. Exit"
		stop_this_shit
	fi
}

server_check() {
	if ! $ssh "$1" true &>/dev/null; then
		echo "Server "$1" is not reachable by SSH public key."
		stop_this_shit
	fi

	if ! $ssh "$1" jq --version &>/dev/null; then
		echo "Server "$1" does not have jq! Please install it first!"
		stop_this_shit
	fi

	if ! $ssh "$1" rsync --version &>/dev/null; then
		echo "Server "$1" does not have rsync! Please install it first!"
		stop_this_shit
	fi

	if ! $ssh "$1" /usr/bin/time --version &>/dev/null; then
		echo "Server "$1" does not have time! Please install it first!"
		stop_this_shit
	fi

	# if ! $ssh "$1" yq --version &>/dev/null; then
	# 	echo "Server "$1" does not have yq! Please install it first!"
	# 	stop_this_shit
	# fi

	if ! $ssh "$1" [ \$EUID -eq 0 ] &>/dev/null; then
		echo "Please login as root on the server "$1""
		stop_this_shit
	fi

	echo "Server $1 is ready for tests"
}

run_tests() {
	chmod +x tests/compat/*
	rsync -e "$ssh" --quiet -av tests/* "$1":/tmp/yunohost-compat/
	rsync -e "$ssh" --quiet --exclude='*/' -av target/$ARCH/release/yunohost "$1":/tmp/yunohost-compat
	
	declare -a foundTests
	
	for testfile in tests/compat/*.sh; do
		testname="$(basename "$testfile")"
		foundTests+=("$testname")
	done

	ssh -q "$1" bash /tmp/yunohost-compat/__runner.sh "${foundTests[@]}"
}

case "$1" in
	"help" | "--help" | "-h")
		help
		exit 0
		;;
	*)
		build_check "$1"
		server_check "$1"
		run_tests "$1"
		;;
esac
