# MinaCalc Rust Bindings

Bindings Rust pour la bibliothèque MinaCalc C++ de calcul de difficulté pour les jeux de rythme.

## Description

Ce projet fournit des bindings Rust sûrs et idiomatiques pour l'API C++ de MinaCalc, permettant de calculer les scores de difficulté (MSD et SSR) pour les jeux de rythme comme Stepmania.

## Fonctionnalités

- **Calcul MSD** : Scores de difficulté pour tous les taux de musique (0.7x à 2.0x)
- **Calcul SSR** : Scores de difficulté pour un taux et un objectif de score spécifiques
- **Interface Rust idiomatique** : Gestion automatique de la mémoire et gestion d'erreurs
- **Sécurité mémoire** : Utilisation de RAII pour éviter les fuites mémoire

## Installation

### Prérequis

- Rust (version 1.70+)
- Un compilateur C++ compatible (GCC, Clang, ou MSVC)
- Les fichiers source C++ de MinaCalc (`API.h`, `API.cpp`, etc.)

### Compilation

```bash
# Cloner le projet
git clone <repository-url>
cd minacalc-rs

# Compiler le projet
cargo build

# Exécuter les tests
cargo test

# Exécuter l'exemple
cargo run --example basic_usage
```

## Utilisation

### Exemple basique

```rust
use minacalc_rs::{Calc, Note, SkillsetScores};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Créer une instance du calculateur
    let calc = Calc::new()?;
    
    // Créer des données de notes
    let notes = vec![
        Note { notes: 4, row_time: 0.0 },
        Note { notes: 0, row_time: 0.5 },
        Note { notes: 4, row_time: 1.0 },
    ];
    
    // Calculer les scores MSD
    let msd_results = calc.calc_msd(&notes)?;
    
    // Calculer les scores SSR pour 1.0x à 95%
    let ssr_scores = calc.calc_ssr(&notes, 1.0, 95.0)?;
    
    println!("Overall MSD: {:.2}", msd_results.msds[3].overall);
    println!("Overall SSR: {:.2}", ssr_scores.overall);
    
    Ok(())
}
```

### API Principale

#### `Calc`

La structure principale pour effectuer les calculs.

```rust
impl Calc {
    /// Crée une nouvelle instance de calculateur
    pub fn new() -> Result<Self, &'static str>
    
    /// Obtient la version du calculateur
    pub fn version() -> i32
    
    /// Calcule les scores MSD pour tous les taux de musique
    pub fn calc_msd(&self, notes: &[Note]) -> Result<MsdForAllRates, &'static str>
    
    /// Calcule les scores SSR pour un taux et un objectif spécifiques
    pub fn calc_ssr(&self, notes: &[Note], music_rate: f32, score_goal: f32) -> Result<SkillsetScores, &'static str>
}
```

#### `Note`

Représente une note dans le jeu de rythme.

```rust
pub struct Note {
    pub notes: u32,      // Nombre de notes à cette position
    pub row_time: f32,   // Temps de la rangée (en secondes)
}
```

#### `SkillsetScores`

Contient les scores de difficulté pour différents skillsets.

```rust
pub struct SkillsetScores {
    pub overall: f32,
    pub stream: f32,
    pub jumpstream: f32,
    pub handstream: f32,
    pub stamina: f32,
    pub jackspeed: f32,
    pub chordjack: f32,
    pub technical: f32,
}
```

#### `MsdForAllRates`

Contient les scores MSD pour tous les taux de musique (0.7x à 2.0x).

```rust
pub struct MsdForAllRates {
    pub msds: [SkillsetScores; 14], // 14 taux de 0.7x à 2.0x
}
```

## Structure du Projet

```
minacalc-rs/
├── Cargo.toml          # Configuration du projet Rust
├── build.rs            # Script de compilation C++
├── API.h               # Header C++ original
├── API.cpp             # Implémentation C++ originale
├── NoteDataStructures.h # Structures de données C++
├── src/
│   ├── lib.rs          # Point d'entrée de la bibliothèque
│   ├── bindings.rs     # Bindings FFI générés automatiquement
│   └── wrapper.rs      # Interface Rust idiomatique
├── examples/
│   └── basic_usage.rs  # Exemple d'utilisation
└── README.md           # Ce fichier
```

## Gestion des Erreurs

Les fonctions retournent des `Result` avec des messages d'erreur descriptifs :

- `"Failed to create calculator"` : Échec de création du calculateur
- `"No notes provided"` : Aucune note fournie
- `"Music rate must be positive"` : Taux de musique invalide
- `"Score goal must be between 0 and 100"` : Objectif de score invalide

## Tests

Exécutez les tests avec :

```bash
cargo test
```

Les tests incluent :
- Vérification de la version du calculateur
- Tests de conversion de types
- Tests de création d'instances

## Licence

MIT License - voir le fichier LICENSE pour plus de détails.

## Contribution

Les contributions sont les bienvenues ! N'hésitez pas à :
- Signaler des bugs
- Proposer des améliorations
- Soumettre des pull requests

## Remerciements

Ce projet est basé sur la bibliothèque MinaCalc originale développée pour Stepmania.
