# Vendored MinaCalc patches

The distributable patch lives at
`crates/minacalc-sys/patches/configurable-calc.patch`, so it is included when
`minacalc-sys` is published. After replacing the pristine vendored sources in
`crates/minacalc-sys/c_code`, verify the temporary patched build with:

```sh
just patch-minacalc
```

The command does not modify `c_code`. `build.rs` copies those sources into
Cargo's `OUT_DIR`, patches and compiles the copy, generates bindings from it,
then removes the temporary tree. It fails when a new upstream version requires
the patch to be rebased. Run `just test` afterward.
