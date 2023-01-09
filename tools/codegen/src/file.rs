use std::path::{Path, PathBuf};

use anyhow::Result;
use fs_err as fs;
use proc_macro2::TokenStream;

// Inspired by https://stackoverflow.com/a/63904992.
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name[..name.len() - 3].rsplit_once(':').unwrap().1
    }};
}

pub fn workspace_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop(); // codegen
    dir.pop(); // tools
    dir
}

#[track_caller]
fn header(function_name: &str) -> String {
    // rust-analyzer does not respect outer attribute (#[rustfmt::skip]) on
    // a module without a body. So use inner attribute under cfg(rustfmt).
    format!(
        "// This file is @generated by {bin_name}
// ({function_name} function at {file}).
// It is not intended for manual editing.\n
#![cfg_attr(rustfmt, rustfmt::skip)]\n
",
        bin_name = env!("CARGO_BIN_NAME"),
        file = std::panic::Location::caller().file()
    )
}

#[track_caller]
pub fn write(function_name: &str, path: &Path, contents: TokenStream) -> Result<()> {
    write_raw(function_name, path, format_tokens(contents))
}

pub fn format_tokens(contents: TokenStream) -> String {
    prettyplease::unparse(
        &syn::parse2(contents.clone())
            .unwrap_or_else(|e| panic!("{} in:\n---\n{}\n---", e, contents)),
    )
    .replace("crate ::", "crate::")
    .replace(" < ", "<")
    .replace(" >", ">")
}

#[track_caller]
pub fn write_raw(function_name: &str, path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    let mut out = header(function_name).into_bytes();
    out.extend_from_slice(contents.as_ref());
    if path.is_file() && fs::read(path)? == out {
        return Ok(());
    }
    fs::write(path, out)?;
    Ok(())
}
