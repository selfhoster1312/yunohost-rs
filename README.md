# yunohost-rs

**WARNING:** DO NOT USE FOR ANYTHING SERIOUS!

This is a reimplementation of [Yunohost](https://yunohost.org/) structures and algorithms in Rust. This way you can Rewrite It In Rust and make it perfectly fast and correct and never have any bug again (<-- sarcasm included).

<p align="center">
<img alt="LOTR meme: One does not simply Rewrite It In Rust" src="https://camo.githubusercontent.com/a5c2198c5e2c4449cf1289c78c1c03ebd85728f7b662c9ee1f142641486d676e/68747470733a2f2f692e696d67666c69702e636f6d2f31706b3162692e6a7067">
</p>

## Why

It's not the first time i rewrite parts of Yunohost in Rust to make it faster. Another crime against my own sanity i committed was the [ssowat-rs experiment](https://kl.netlib.re/gitea/selfhoster1312/ssowat-rs) which gave me greater understanding to [make the actual new branch of ssowat faster](https://github.com/YunoHost/SSOwat/pull/220). All in all, there's two reasons to rewrite parts of Yunohost in Rust:

- better understand the algorithms and how they can be optimized upstream
- showcase a faster implementation to serve as a benchmark

I also believe the Yunohost project would benefit from writing many things in Rust, in general, but that's not the goal here.

## Architecture

You will find:

- `hooks`: for now only regen-conf hooks in there
- `src/helpers/`: high-level utilities to make your life easier
- `src/*.rs`: the yunohost subcommands that can be built as independant binaries
- `src/lib.rs`: the top of the library
- `tests/compat/*.sh`: each file contains tests for a specific yunohost command
- `test.sh`: the main script triggering the test suite

## Try me out

If you'd like to try a single tool, you can build it with `yunohost-` prepended, like so: `cargo build --release yunohost-user`. To build all the tools at once, you can do `cargo build --release`. The binaries are found in `target/x86_64-unknown-linux-musl`. They are targeting the [musl](https://en.wikipedia.org/wiki/Musl) so that they work magically across [glibc](https://en.wikipedia.org/wiki/Glibc) versions, for example when compiling from Archlinux or Debian Bookworm.

**Warning:** Don't forget the `--release` flag. Using rust binaries in debug mode can lead to very slow code and crashes.

## Test me

To run the unit tests (there aren't a lot yet), run `cargo test --release`. The `--release` so that you don't have to recompile everything when trying the binaries for real.

To run the integration test, give access for your machine's SSH key to a remote Yunohost server, then run `./test.sh SERVERNAME`. It will produce an output like this:

```
Server kl.netlib.re is ready for tests
__runner.sh hook.sh tools.sh user.sh
DIFF                        PYTHON      RUST                         COMMAND
DIFF - /tmp/tmp.NJ33lvt6jQ  OK (0.32s)  OK (0.00s)  yunohost hook list --json conf_regen
DIFF - /tmp/tmp.dfIKQkmask  OK (0.40s)  OK (0.00s)  yunohost settings get --json security
DIFF - /tmp/tmp.R2q2z26rc2  OK (0.36s)  OK (0.00s)  yunohost settings get --json security.webadmin
OK                          OK (0.28s)  OK (0.00s)  yunohost settings get --json security.webadmin.webadmin_allowlist_enabled
DIFF - /tmp/tmp.qwZLB3DCjg  OK (0.51s)  OK (0.01s)  yunohost tools regen-conf --list-pending --json
OK                          OK (0.31s)  OK (0.01s)  yunohost user list --json
DIFF - /tmp/tmp.eDk6n9ULyR  OK (0.82s)  OK (0.00s)  yunohost user info --json test2
```

Some notes about integration tests:

- for now we only test read-only operations
- it requires that an account named `test2` exists on the Yunohost server
- it compares success/timing between python and rust version ; an error in execution will show the path to the logged command output
- it compares JSON output between python and rust version ; if the outputs don't match after normalization (via jq) then it shows a path to the logged (normalized) diff

Just one note about Rust execution speed: **YES IT IS THAT FAST**. Sometimes when the server is busy it will climb to 0.01s for one command, otherwise `time -f %e` is not capable to time it properly. I'm fine with that, this is not a scientific benchmark it's just to make sure it's not horribly slow because of a mistake.

## TODO

- [x] proper error management
- [ ] paste.yunohost.org integration on error
- [x] rewrite one regen-conf in Rust ([done](src/hooks/01-yunohost.rs))
- [x] rewrite `yunohost user list` in Rust (TODO: `--fields` argument)
- [ ] rewrite the regen-conf engine in Rust
- [ ] rewrite all regen-conf engine/hooks in Rust
- [~] rewrite `yunohost settings get` in Rust (initial very incomplete implementation)
- [x] output comparison in integration tests
- [x] timing information in integration tests
- [ ] Github Pages crate documentation
- [x] Utf8Path (camino) integration for easier Path<->str interop
- [ ] publish Debian repo on Github pages so people can test it without compiling
- [ ] rewrite moulinette.i18n for translations
- [ ] rewrite moulinette actionsmap parser and integrate with real Yunohost
