fn main() {
  println!("cargo::rerun-if-changed=build.rs");

  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") && std::env::var("PROFILE").as_deref() == Ok("debug") {
    // Debug integration tests exceed the 16 MiB compact-unwind offset limit;
    // keep DWARF unwinding for tests without changing release binaries.
    println!("cargo::rustc-link-arg-tests=-Wl,-no_compact_unwind");
  }
}
