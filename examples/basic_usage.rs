use minacalc_rs::{Calc, Note};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MinaCalc Rust Bindings - Exemple d'utilisation");
    println!("Version du calculateur: {}", Calc::version());

    // Créer une instance du calculateur
    let calc = Calc::new()?;

    // Créer des données de notes d'exemple
    // Ces notes représentent un pattern simple
    // Format: notes est un bitflag où chaque bit représente une colonne
    // 0b0001 = colonne 1, 0b0010 = colonne 2, 0b0100 = colonne 3, 0b1000 = colonne 4
    // 0b0011 = colonnes 1+2 (jump), 0b1111 = toutes les colonnes (quad)
    let all_notes = vec![
        Note {
            notes: 0b0001,
            row_time: 0.0,
        }, // Single note colonne 1
        Note {
            notes: 0b0010,
            row_time: 0.25,
        }, // Single note colonne 2
        Note {
            notes: 0b0100,
            row_time: 0.5,
        }, // Single note colonne 3
        Note {
            notes: 0b1000,
            row_time: 0.75,
        }, // Single note colonne 4
        Note {
            notes: 0b0011,
            row_time: 1.0,
        }, // Jump (2 notes)
        Note {
            notes: 0b0101,
            row_time: 1.25,
        }, // Jump (2 notes)
        Note {
            notes: 0b1010,
            row_time: 1.5,
        }, // Jump (2 notes)
        Note {
            notes: 0b0111,
            row_time: 2.0,
        }, // Hand (3 notes)
        Note {
            notes: 0b1110,
            row_time: 2.25,
        }, // Hand (3 notes)
        Note {
            notes: 0b1111,
            row_time: 2.5,
        }, // Quad (4 notes)
        Note {
            notes: 0b0001,
            row_time: 3.0,
        }, // Single note
        Note {
            notes: 0b0010,
            row_time: 3.1,
        }, // Single note rapide
        Note {
            notes: 0b0001,
            row_time: 3.2,
        }, // Jack sur colonne 1
        Note {
            notes: 0b0010,
            row_time: 3.3,
        }, // Single note
    ];

    // Filtrer les notes vides (notes == 0) car MinaCalc ne les accepte pas
    let notes: Vec<Note> = all_notes.into_iter().filter(|n| n.notes != 0).collect();

    println!("Nombre de notes actives: {}\n", notes.len());

    println!("Calcul des scores MSD pour tous les taux de musique...");

    // Calculer les scores MSD pour tous les taux (0.7x à 2.0x)
    match calc.calc_msd(&notes) {
        Ok(msd_results) => {
            println!("Scores MSD calculés avec succès!\n");

            // Afficher les scores pour quelques taux
            let rates = [0.7, 1.0, 1.5, 2.0];
            let rate_indices = [0, 3, 8, 13]; // Indices correspondants dans le tableau

            for (rate, &index) in rates.iter().zip(rate_indices.iter()) {
                let scores = msd_results.msds[index];
                println!("Taux {:.1}x:", rate);
                println!("  Overall: {:.2}", scores.overall);
                println!("  Stream: {:.2}", scores.stream);
                println!("  Jumpstream: {:.2}", scores.jumpstream);
                println!("  Handstream: {:.2}", scores.handstream);
                println!("  Stamina: {:.2}", scores.stamina);
                println!("  Jack Speed: {:.2}", scores.jackspeed);
                println!("  Chordjack: {:.2}", scores.chordjack);
                println!("  Technical: {:.2}", scores.technical);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Erreur lors du calcul MSD: {}", e);
        }
    }

    println!("Calcul des scores SSR pour un taux spécifique...");

    // Calculer les scores SSR pour un taux de 1.0x avec un objectif de 93%
    match calc.calc_ssr(&notes, 1.0, 93.0) {
        Ok(ssr_scores) => {
            println!("Scores SSR calculés avec succès pour 1.0x à 93%!");
            println!("Overall: {:.2}", ssr_scores.overall);
            println!("Stream: {:.2}", ssr_scores.stream);
            println!("Jumpstream: {:.2}", ssr_scores.jumpstream);
            println!("Handstream: {:.2}", ssr_scores.handstream);
            println!("Stamina: {:.2}", ssr_scores.stamina);
            println!("Jack Speed: {:.2}", ssr_scores.jackspeed);
            println!("Chordjack: {:.2}", ssr_scores.chordjack);
            println!("Technical: {:.2}", ssr_scores.technical);
        }
        Err(e) => {
            eprintln!("Erreur lors du calcul SSR: {}", e);
        }
    }

    Ok(())
}
