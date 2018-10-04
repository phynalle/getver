# getver
getver is a simple command line tool for capturing the latest version of crates.
```
$ getver <crate>...
```

### Prerequsite
getver needs stable version of rust. [Here](https://rustup.rs/) provides the easy way to install rust
```
$ curl https://sh.rustup.rs -sSf | sh
$ rustup install stable
```

### Installation
```
$ cargo install getver
```

after installation, try this:
```
$ getver libc bitflags rand log lazy_static
```
