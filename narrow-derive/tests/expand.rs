#[test]
#[cfg(narrow_macrotest)]
/// To run these tests:
/// `RUSTFLAGS='--cfg narrow_macrotest' cargo test -p narrow-derive expand`
///
/// To update the generated output:
/// `MACROTEST=overwrite RUSTFLAGS='--cfg narrow_macrotest' cargo test -p narrow-derive expand`
fn expand() {
    macrotest::expand("tests/expand/**/*.rs");
}
