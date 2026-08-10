# Releasing MinaCalc packages

Pushing a tag matching the Rust crate version, for example `v515.2.0`, runs
`.github/workflows/release.yml`. The workflow validates every package before
publishing anything, then performs these releases:

- `minacalc-sys`, `minacalc-rs`, and `minacalc-bindings` on crates.io;
- `@glubus/minacalc` on npm;
- `minacalc` on PyPI;
- `Glubus.MinaCalc` on NuGet;
- `bindings/go/v0.2.0`, which publishes the Go submodule;
- native Linux, macOS, and Windows libraries on the GitHub Release.

## One-time repository setup

Create the GitHub environments `crates-io`, `npm`, `pypi`, `nuget`, and `go`.
Required secrets are:

| Secret | Used for |
| --- | --- |
| `CARGO_REGISTRY_TOKEN` | crates.io |
| `NPM_TOKEN` | npm, with publish access to the `@glubus` scope |
| `NUGET_API_KEY` | NuGet package `Glubus.MinaCalc` |

PyPI uses short-lived OIDC credentials rather than a stored token. Configure a
PyPI Trusted Publisher for repository `Glubus/minacalc-rs`, workflow
`release.yml`, environment `pypi`, and project `minacalc`.

The workflow uses the tag `v515.Y.0` for Rust packages and derives wrapper
version `0.Y.0`. It refuses to publish when Cargo, npm, Python, and .NET versions
do not match that convention.

## Release command

Run the complete local QA, then push the tag:

```sh
just check
git tag -a v515.2.0 -m "Release v515.2.0"
git push origin v515.2.0
```

The Go tag is created by CI. Do not create it manually. Protected environments
with required reviewers are recommended for every publishing job.
