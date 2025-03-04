use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::{env, io, process, str};

use walkdir::WalkDir;

static HEADER: &str = "#include <libsmartcols/libsmartcols.h>";
static HEADER_DOCS_RS: &str = r#"#include "src/libsmartcols.h""#;

fn main() {
    let target =
        env::var("TARGET").expect("smartcols-sys: Environment variable 'TARGET' was not defined");

    if !target.contains("-linux-") || target.ends_with("-kernel") {
        return; // Nothing to build for this architecture.
    }

    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .expect("smartcols-sys: Environment variable 'OUT_DIR' was not defined");

    println!("cargo:root={}", path_to_str(&out_dir));

    let sysroot = target_env_var_os("SYSROOT", &target).map(PathBuf::from);

    let explicit_static = get_static_linking(&target);
    let compiler_search_paths = get_compiler_search_paths(&target, sysroot.as_deref());

    let docs_only_mode = Some(OsStr::new("1")) == env::var_os("DOCS_RS").as_deref();
    if docs_only_mode {
        println!(
            "cargo:warning=smartcols-sys: Documentation-only mode. Generated crate might be UNUSABLE."
        );
    }

    let include_path =
        find_and_output_include_dir(&compiler_search_paths.include_paths, docs_only_mode);

    find_and_output_lib_dir(
        &compiler_search_paths.link_paths,
        &target,
        explicit_static,
        docs_only_mode,
    );

    generate_bindings(
        &target,
        sysroot.as_deref(),
        &out_dir,
        &include_path,
        if docs_only_mode {
            HEADER_DOCS_RS
        } else {
            HEADER
        },
    )
}

fn path_to_str(path: &Path) -> &str {
    path.to_str()
        .expect("smartcols-sys: Path is not valid UTF-8")
}

#[cfg(feature = "static")]
fn get_static_linking(target: &str) -> Option<bool> {
    target_env_var_os("SMARTCOLS_STATIC", target)
        .map(|v| v == "1" || v == "true")
        .or_else(|| Some(true))
}

#[cfg(not(feature = "static"))]
fn get_static_linking(target: &str) -> Option<bool> {
    target_env_var_os("SMARTCOLS_STATIC", target).map(|v| v == "1" || v == "true")
}

fn get_compiler_search_paths(target: &str, sysroot: Option<&Path>) -> CompilerSearchPaths {
    let explicit_path = target_env_var_os("SMARTCOLS_PATH", target).map(PathBuf::from);

    let mut include_dir = target_env_var_os("SMARTCOLS_INCLUDE_DIR", target).map(PathBuf::from);
    let mut link_dir = target_env_var_os("SMARTCOLS_LIB_DIR", target).map(PathBuf::from);

    for &name in &["CC", "CFLAGS"] {
        target_env_var_os(name, target);
    }

    if let Some(explicit_path) = explicit_path {
        if include_dir.is_none() {
            include_dir = Some(explicit_path.join("include"));
        }

        if link_dir.is_none() {
            link_dir = Some(explicit_path.join("lib"));
        }
    }

    CompilerSearchPaths::new(sysroot, include_dir, link_dir)
}

#[derive(Debug)]
struct CompilerSearchPaths {
    include_paths: Vec<PathBuf>,
    link_paths: Vec<PathBuf>,
}

impl CompilerSearchPaths {
    fn new(
        sysroot: Option<&Path>,
        include_dir: Option<PathBuf>,
        link_dir: Option<PathBuf>,
    ) -> Self {
        unsafe { env::set_var("LANG", "C") };

        let include_paths = Self::get_compiler_include_paths(sysroot, include_dir)
            .expect("smartcols-sys: Failed to discover default compiler search paths");

        let link_paths = Self::get_compiler_link_paths(sysroot, link_dir)
            .expect("smartcols-sys: Failed to discover default linker search paths");

        CompilerSearchPaths {
            include_paths,
            link_paths,
        }
    }

    fn get_compiler_include_paths(
        sysroot: Option<&Path>,
        include_dir: Option<PathBuf>,
    ) -> io::Result<Vec<PathBuf>> {
        let mut compiler_builder = cc::Build::new();

        if let Some(sysroot) = sysroot.map(Path::as_os_str).map(OsStr::to_str) {
            let sysroot = sysroot.expect("SYSROOT is not encoded in UTF-8");
            compiler_builder.flag(format!("--sysroot={sysroot}"));
        }

        if let Some(include_dir) = include_dir.as_deref() {
            compiler_builder.include(include_dir);
        }

        let child = compiler_builder
            .flag("-E")
            .flag("-v")
            .flag("-x")
            .flag("c")
            .get_compiler()
            .to_command()
            .arg("-") // stdin
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::piped())
            .env("LANG", "C")
            .spawn()?;

        let output = child.wait_with_output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Compiler failed to print search directories",
            ));
        }

        let mut paths = Vec::with_capacity(8);

        if let Some(include_dir) = include_dir {
            paths.push(include_dir);
        }

        paths.extend(
            output
                .stderr
                .split(|&b| b == b'\n')
                .skip_while(|&line| line != b"#include <...> search starts here:")
                .take_while(|&line| line != b"End of search list.")
                .filter_map(|bytes| str::from_utf8(bytes).ok())
                .map(str::trim)
                .filter_map(|s| dunce::canonicalize(s).ok()),
        );

        paths.dedup();
        Ok(paths)
    }

    fn get_compiler_link_paths(
        sysroot: Option<&Path>,
        link_dir: Option<PathBuf>,
    ) -> io::Result<Vec<PathBuf>> {
        let mut compiler_builder = cc::Build::new();

        if let Some(sysroot) = sysroot.map(Path::as_os_str).map(OsStr::to_str) {
            let sysroot = sysroot.expect("SYSROOT is not encoded in UTF-8");
            compiler_builder.flag(format!("--sysroot={sysroot}"));
        }

        if let Some(link_dir) = link_dir.as_deref() {
            compiler_builder.flag("-L").flag(path_to_str(link_dir));
        }

        let child = compiler_builder
            .flag("-v")
            .flag("-print-search-dirs")
            .get_compiler()
            .to_command()
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::null())
            .env("LANG", "C")
            .spawn()?;

        let output = child.wait_with_output()?;

        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Compiler failed to print search directories",
            ));
        }

        let line = output
            .stdout
            .split(|&b| b == b'\n')
            .find_map(|line| line.strip_prefix(b"libraries:"))
            .and_then(|bytes| str::from_utf8(bytes).ok())
            .map(str::trim)
            .map(|line| line.trim_start_matches('='))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Compiler search directories format is unrecognized",
                )
            })?;

        let mut paths = Vec::with_capacity(8);

        if let Some(link_dir) = link_dir {
            paths.push(link_dir);
        }

        if let Some(lib_paths) = env::var_os("LIBRARY_PATH") {
            paths.extend(env::split_paths(&lib_paths).filter_map(|s| dunce::canonicalize(s).ok()));
        }

        paths.extend(env::split_paths(line).filter_map(|s| dunce::canonicalize(s).ok()));

        paths.dedup();
        Ok(paths)
    }
}

fn target_env_var_os(name: &str, target: &str) -> Option<OsString> {
    rerun_if_env_changed(name, target);

    let target_underscores = target.replace('-', "_");

    env::var_os(format!("{name}_{target}"))
        .or_else(|| env::var_os(format!("{name}_{target_underscores}")))
        .or_else(|| env::var_os(format!("TARGET_{name}")))
        .or_else(|| env::var_os(name))
}

fn rerun_if_env_changed(name: &str, target: &str) {
    let target_underscores = target.replace('-', "_");

    println!("cargo:rerun-if-env-changed={name}_{target}");
    println!("cargo:rerun-if-env-changed={name}_{target_underscores}");
    println!("cargo:rerun-if-env-changed=TARGET_{name}");
    println!("cargo:rerun-if-env-changed={name}");
}

fn rerun_if_dir_changed(dir: &Path, must_exist: bool) {
    for file in WalkDir::new(dir).follow_links(false).same_file_system(true) {
        if let Ok(file) = file {
            println!("cargo:rerun-if-changed={}", file.path().display());
        } else if must_exist {
            panic!(
                "smartcols-sys: Failed to list directory contents: {}",
                dir.display()
            );
        }
    }
}

fn find_and_output_include_dir(include_paths: &[PathBuf], docs_only_mode: bool) -> PathBuf {
    let include_path = find_file_in_dirs("libsmartcols/libsmartcols.h", include_paths);

    let include_path = if docs_only_mode {
        include_path.unwrap_or_else(|_| PathBuf::from("/inexistent"))
    } else {
        include_path.expect(
            "smartcols-sys: Failed to find 'libsmartcols/libsmartcols.h'. \
             Please make sure the C header files of libsmartcols are installed and accessible",
        )
    };

    rerun_if_dir_changed(&include_path.join("libsmartcols"), !docs_only_mode);

    println!("cargo:include={}", path_to_str(&include_path));

    include_path
}

fn output_lib_dir(dir: &Path, file: &Path, static_lib: bool) {
    println!("cargo:rerun-if-changed={}", file.display());

    println!("cargo:lib={}", path_to_str(dir));

    println!("cargo:rustc-link-search=native={}", path_to_str(dir));

    println!(
        "cargo:rustc-link-lib={}=smartcols",
        if static_lib { "static" } else { "dylib" }
    );
}

fn find_and_output_lib_dir(
    link_paths: &[PathBuf],
    target: &str,
    explicit_static: Option<bool>,
    docs_only_mode: bool,
) {
    let lib_configs = match explicit_static {
        Some(false) => vec![false],

        Some(true) => vec![true],

        None => {
            if target.contains("-musl") {
                vec![true, false]
            } else {
                vec![false, true]
            }
        }
    };

    for &static_lib in &lib_configs {
        let file_name = format!("libsmartcols{}", if static_lib { ".a" } else { ".so" });

        if let Ok(lib_path) = find_file_in_dirs(&file_name, link_paths) {
            output_lib_dir(&lib_path, &lib_path.join(&file_name), static_lib);
            return;
        }

        if let Some(link_path) = link_paths.first() {
            let triplet = target.replace("-unknown-", "-").replace("-none-", "-");

            for &lib_dir in &[link_path, &link_path.join(target), &link_path.join(triplet)] {
                let lib_path = lib_dir.join(&file_name);
                if let Ok(md) = lib_path.metadata() {
                    if md.is_file() {
                        output_lib_dir(lib_dir, &lib_path, static_lib);
                        return;
                    }
                }
            }
        }
    }

    if docs_only_mode {
        output_lib_dir(
            Path::new("/inexistent"),
            Path::new("/inexistent"),
            lib_configs[0],
        );
    }
}

// See: https://github.com/rust-lang/rust-bindgen/issues/2136
fn translate_rustc_target_to_clang(rustc_target: &str) -> Cow<str> {
    if let Some(suffix) = rustc_target.strip_prefix("riscv32") {
        let suffix = suffix.trim_start_matches(|c| c != '-');
        Cow::Owned(format!("riscv32{suffix}"))
    } else if let Some(suffix) = rustc_target.strip_prefix("riscv64") {
        let suffix = suffix.trim_start_matches(|c| c != '-');
        Cow::Owned(format!("riscv64{suffix}"))
    } else if let Some(suffix) = rustc_target.strip_prefix("aarch64-apple-") {
        Cow::Owned(format!("arm64-apple-{suffix}"))
    } else if let Some(prefix) = rustc_target.strip_suffix("-espidf") {
        Cow::Owned(format!("{prefix}-elf"))
    } else {
        Cow::Borrowed(rustc_target)
    }
}

fn generate_bindings(
    target: &str,
    sysroot: Option<&Path>,
    out_dir: &Path,
    include_path: &Path,
    header: &str,
) {
    let clang_target = translate_rustc_target_to_clang(target);

    let mut builder = bindgen::Builder::default()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .default_enum_style(bindgen::EnumVariation::Consts)
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .size_t_is_usize(true)
        .derive_debug(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_ord(true)
        .impl_debug(true)
        .clang_arg(format!("--target={clang_target}"))
        .clang_args(&["-I", path_to_str(include_path)]);

    if let Some(sysroot) = sysroot.map(Path::as_os_str).map(OsStr::to_str) {
        let sysroot = sysroot.expect("SYSROOT is not encoded in UTF-8");
        builder = builder.clang_arg(format!("--sysroot={sysroot}"));
    }

    // Make the `FILE` type opaque, so bindgen does not pull other types from the standard library.
    // Then block `FILE`, so bindgen does not emit it.
    // Then define `FILE` explicitly so that it can only be used behind a pointer.
    builder = builder.opaque_type("^FILE$");
    builder = builder.blocklist_type("^FILE$");
    builder = builder.raw_line("pub type FILE = ::std::os::raw::c_void;");

    builder = builder.allowlist_type("^libscols_.+$");
    for name in ["^LIBSMARTCOLS_.+$", "^SCOLS_.+$"] {
        builder = builder.allowlist_var(name);
    }
    builder = builder.allowlist_function("^scols_.+$");
    builder = builder.header_contents("smartcols-sys.h", header);

    let bindings = builder.generate().expect(
        "smartcols-sys: Failed to generate Rust bindings for 'libsmartcols/libsmartcols.h' and other headers",
    );

    bindings
        .write_to_file(out_dir.join("smartcols-sys.rs"))
        .expect("smartcols-sys: Failed to write 'smartcols-sys.rs'")
}

fn find_file_in_dirs(path_suffix: &str, dirs: &[PathBuf]) -> io::Result<PathBuf> {
    for dir in dirs {
        if let Ok(md) = dir.join(path_suffix).metadata() {
            if md.file_type().is_file() {
                return Ok(dir.clone());
            }
        }
    }

    Err(io::ErrorKind::NotFound.into())
}
