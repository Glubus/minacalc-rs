use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct PatchedSource {
    root: PathBuf,
    c_code: PathBuf,
}

impl PatchedSource {
    fn prepare(manifest_dir: &Path, out_dir: &Path) -> Self {
        let root = out_dir.join("patched-minacalc-source");
        if root.exists() {
            fs::remove_dir_all(&root).expect("failed to clear stale patched MinaCalc sources");
        }

        let c_code = root.join("crates/minacalc-sys/c_code");
        copy_dir(&manifest_dir.join("c_code"), &c_code)
            .expect("failed to copy MinaCalc sources into target");

        let patch = manifest_dir.join("patches/configurable-calc.patch");
        let output = Command::new("git")
            // Force `git apply` into no-repository mode. OUT_DIR is commonly
            // below the workspace, so repository discovery would otherwise
            // patch the checked-in c_code instead of this temporary copy.
            .env("GIT_DIR", root.join("nonexistent.git"))
            .args(["apply", "--unsafe-paths", "--whitespace=nowarn"])
            .arg(&patch)
            .current_dir(&root)
            .output()
            .expect("failed to execute `git apply` for MinaCalc sources");
        if !output.status.success() {
            panic!(
                "failed to patch temporary MinaCalc sources:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Self { root, c_code }
    }
}

impl Drop for PatchedSource {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("failed to remove patched MinaCalc sources");
    }
}

fn copy_dir(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let patched = PatchedSource::prepare(&manifest_dir, &out_path);

    // Compile only the temporary patched copy under target/.
    let mut build = cc::Build::new();

    // Ajouter les fichiers source C++
    build
        .cpp(true)
        .file(patched.c_code.join("API.cpp"))
        .file(patched.c_code.join("Guarded.cpp"))
        .file(patched.c_code.join("MinaCalc/MinaCalc.cpp"))
        .include(&patched.c_code)
        .include(patched.c_code.join("MinaCalc"));

    // Détecter le compilateur et ajouter les flags appropriés
    let target = env::var("TARGET").unwrap_or_default();
    build.define("STANDALONE_CALC", None);
    // As in Etterna's release builds: a failed C `assert` calls abort(), which no try/catch stops, and cc never defines
    // NDEBUG by itself, so without it a debug check of MinaCalc could end the whole program in every profile.
    build.define("NDEBUG", None);
    if target.contains("msvc") {
        build.flag("/std:c++20");
        // C++ exceptions are caught in Guarded.cpp: the unwind tables must exist (cc may not add /EHsc by itself).
        build.flag("/EHsc");
        build.flag("/W0");
    } else {
        build.flag("-std=c++20");
        build.flag("-w");
    }

    // Compiler la bibliothèque
    build.compile("minacalc");

    // Générer les bindings FFI
    let bindings = bindgen::Builder::default()
        .header(patched.c_code.join("Guarded.h").to_string_lossy())
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++20")
        .rustified_enum("CalcMode")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Écrire les bindings dans le répertoire de sortie
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Recompile when any MinaCalc source changes (Cargo scans the directories recursively).
    println!("cargo:rerun-if-changed=c_code");
    println!("cargo:rerun-if-changed=patches/configurable-calc.patch");

    // Définir des types conditionnels pour unsigned long
    // println!("cargo:rustc-cfg=target_os=\"{}\"", env::var("CARGO_CFG_TARGET_OS").unwrap());
}
