use std::env;
use std::path::PathBuf;

fn main() {
    // Compiler le code C++
    let mut build = cc::Build::new();
    
    // Ajouter les fichiers source C++
    build.cpp(true)
          .file("API.cpp")
          .file("MinaCalc/MinaCalc.cpp")
          .include(".")
          .include("MinaCalc")
          .flag("-std=c++17");
    
    // Compiler la bibliothèque
    build.compile("minacalc");
    
    // Générer les bindings FFI
    let bindings = bindgen::Builder::default()
        .header("API.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    
    // Écrire les bindings dans le répertoire de sortie
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    
    // Indiquer à Cargo de recompiler si les fichiers C++ changent
    println!("cargo:rerun-if-changed=API.h");
    println!("cargo:rerun-if-changed=API.cpp");
    println!("cargo:rerun-if-changed=NoteDataStructures.h");
    println!("cargo:rerun-if-changed=MinaCalc/MinaCalc.cpp");
}
