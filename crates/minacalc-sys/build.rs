use std::env;
use std::path::PathBuf;

fn main() {
    // Compiler le code C++
    let mut build = cc::Build::new();

    // Ajouter les fichiers source C++
    build
        .cpp(true)
        .file("c_code/API.cpp")
        .file("c_code/MinaCalc/MinaCalc.cpp")
        .include("c_code")
        .include("c_code/MinaCalc");

    // Détecter le compilateur et ajouter les flags appropriés
    let target = env::var("TARGET").unwrap_or_default();
    build.define("STANDALONE_CALC", None);
    if target.contains("msvc") {
        build.flag("/std:c++20");
        build.flag("/W0");
    } else {
        build.flag("-std=c++20");
        build.flag("-w");
    }

    // Compiler la bibliothèque
    build.compile("minacalc");

    // Générer les bindings FFI
    let bindings = bindgen::Builder::default()
        .header("c_code/API.h")
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++20")
        .rustified_enum("CalcMode")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Écrire les bindings dans le répertoire de sortie
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Indiquer à Cargo de recompiler si les fichiers C++ changent
    println!("cargo:rerun-if-changed=c_code/API.h");
    println!("cargo:rerun-if-changed=c_code/API.cpp");
    println!("cargo:rerun-if-changed=c_code/Models/NoteData/NoteDataStructures.h");
    println!("cargo:rerun-if-changed=c_code/MinaCalc/MinaCalc.cpp");
    println!("cargo:rerun-if-changed=c_code/MinaCalc/MinaCalc.h");
    println!("cargo:rerun-if-changed=c_code/MinaCalc/UlbuAcolytes.h");
    println!("cargo:rerun-if-changed=c_code/MinaCalc/UlbuBase.h");
    println!("cargo:rerun-if-changed=c_code/MinaCalc/UlbuSevenKey.h");
    println!("cargo:rerun-if-changed=c_code/MinaCalc/UlbuSixKey.h");
    println!("cargo:rerun-if-changed=c_code/MinaCalc/Ulbu.h");
    println!("cargo:rerun-if-changed=c_code/MinaCalc/SequencingHelpers.h");
    println!("cargo:rerun-if-changed=c_code/MinaCalc/Agnostic/IntervalInfo.h");

    // Définir des types conditionnels pour unsigned long
    // println!("cargo:rustc-cfg=target_os=\"{}\"", env::var("CARGO_CFG_TARGET_OS").unwrap());
}
