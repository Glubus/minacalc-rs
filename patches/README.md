# Vendored MinaCalc patches

After replacing the vendored sources in `crates/minacalc-sys/c_code/MinaCalc`,
reapply the minacalc-rs configurable calculator settings with:

```sh
just patch-minacalc
```

The command is idempotent. It exits successfully when the patch is already
present and fails with an explicit message when a new upstream version needs a
manual conflict resolution. Run `just test` after applying it.
