[![crates.io](https://img.shields.io/crates/v/smartcols-sys.svg)](https://crates.io/crates/smartcols-sys)
[![docs.rs](https://docs.rs/smartcols-sys/badge.svg)](https://docs.rs/smartcols-sys)
[![license](https://img.shields.io/github/license/mdcssw/smartcols-sys?color=000000)](https://raw.githubusercontent.com/mdcssw/smartcols-sys/master/LICENSE.txt)

# `smartcols-sys`: Unsafe Rust bindings for `libsmartcols`

`libsmartcols` helps format tables when structured data is output.

This crate is Linux-specific. Building it for non-Linux platforms, or for
the Linux kernel, results in an empty crate.

This crate links to [`libsmartcols`], and requires it to be installed.
See below for example installation instructions.

## Installing `libsmartcols`

This crate links to [`libsmartcols`], and requires it to be installed.

The library, its C header files, and the `clang` compiler need to be installed on the **build machine**.
On Debian, for example, one can install that as follows:

```sh
sudo apt-get install clang libsmartcols-dev
```

If the library is linked dynamically (most typical configuration), then it needs to be installed
on the target computer in order to run the resulting program.
On Debian, for example, one can install it as follows:

```sh
sudo apt-get install libsmartcols1
```

## Supported environment variables

This crate depends on some environment variables, and *variants* of those.
For each environment variable (e.g., `CC`), the following are the accepted
variants of it:

- `<var>_<target>`, *e.g.,* `CC_aarch64-unknown-linux-gnu`.
- `<var>_<target-with-underscores>`, *e.g.,* `CC_aarch64_unknown_linux_gnu`.
- `TARGET_<var>`, *e.g.,* `TARGET_CC`.
- `<var>`, *e.g.,* `CC`.

The following environment variables (and their variants) affect how this crate
is built:

- `SMARTCOLS_STATIC`
- `SMARTCOLS_PATH`
- `SMARTCOLS_INCLUDE_DIR`
- `SMARTCOLS_LIB_DIR`
- `SYSROOT`
- `CC`
- `CFLAGS`

## Dynamic or static linking

This crate links to `libsmartcols` dynamically if possible, except when targeting
platforms based on the `musl` C library.

This behavior can be changed either by setting the environment variable
`SMARTCOLS_STATIC` to `1`, or by enabling the crate feature `static`.
If both are defined, then the value of `SMARTCOLS_STATIC` takes precedence.

Setting `SMARTCOLS_STATIC` to `0` mandates dynamic linking.

## Finding smartcols library and headers

By default, this crate finds smartcols headers and library based on the default
target C compiler.

This behavior can be changed by:

- Either defining the environment variable `SMARTCOLS_PATH` to the path of
  a directory containing the sub-directories `include` and `lib` where
  the headers and library are installed.
- Or by defining one or both of the environment variables `SMARTCOLS_INCLUDE_DIR`
  and `SMARTCOLS_LIB_DIR` to paths to the directories where headers and library
  are present. If `SMARTCOLS_PATH` is also defined, then `SMARTCOLS_INCLUDE_DIR`
  and `SMARTCOLS_LIB_DIR` take precedence.

## Depending on this crate

This crate provides the following variables to other crates that depend on it:

- `DEP_SMARTCOLS_INCLUDE`: Path of the directory where library C header files reside.
- `DEP_SMARTCOLS_LIB`: Path of the directory where the library binary resides.

## Documentation-only build mode

The *documentation-only* build mode allows building documentation even if
`libsmartcols` and its headers are unavailable.
To build in this mode, set the environment variable `DOCS_RS` to `1`:

```bash
env DOCS_RS=1 cargo doc --open
```

The generated documentation is based on `libsmartcols` version `2.40.4`.

> ⚠️ The generated crate might be **unusable** in this mode.

## Versioning

This project adheres to [Semantic Versioning].
The `CHANGELOG.md` file details notable changes over time.

[Semantic Versioning]: https://semver.org/spec/v2.0.0.html
[`libsmartcols`]: https://github.com/util-linux/util-linux/tree/master/libsmartcols
