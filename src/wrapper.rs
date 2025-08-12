use crate::{NoteInfo, Ssr, MsdForAllRates as BindingsMsdForAllRates, CalcHandle, create_calc, calc_version, calc_msd, calc_ssr, destroy_calc};

/// Représente une note dans le jeu de rythme
#[derive(Debug, Clone, Copy)]
pub struct Note {
    /// Nombre de notes à cette position temporelle
    pub notes: u32,
    /// Temps de la rangée (en secondes)
    pub row_time: f32,
}

impl From<Note> for NoteInfo {
    fn from(note: Note) -> Self {
        NoteInfo {
            notes: note.notes,
            rowTime: note.row_time,
        }
    }
}

impl From<NoteInfo> for Note {
    fn from(note_info: NoteInfo) -> Self {
        Note {
            notes: note_info.notes,
            row_time: note_info.rowTime,
        }
    }
}

/// Représente les scores de difficulté pour différents skillsets
#[derive(Debug, Clone, Copy)]
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

impl From<Ssr> for SkillsetScores {
    fn from(ssr: Ssr) -> Self {
        SkillsetScores {
            overall: ssr.overall,
            stream: ssr.stream,
            jumpstream: ssr.jumpstream,
            handstream: ssr.handstream,
            stamina: ssr.stamina,
            jackspeed: ssr.jackspeed,
            chordjack: ssr.chordjack,
            technical: ssr.technical,
        }
    }
}

impl From<SkillsetScores> for Ssr {
    fn from(scores: SkillsetScores) -> Self {
        Ssr {
            overall: scores.overall,
            stream: scores.stream,
            jumpstream: scores.jumpstream,
            handstream: scores.handstream,
            stamina: scores.stamina,
            jackspeed: scores.jackspeed,
            chordjack: scores.chordjack,
            technical: scores.technical,
        }
    }
}

/// Représente les scores MSD pour tous les taux de musique (0.7x à 2.0x)
#[derive(Debug, Clone)]
pub struct MsdForAllRates {
    pub msds: [SkillsetScores; 14],
}

impl From<MsdForAllRates> for super::MsdForAllRates {
    fn from(msd: MsdForAllRates) -> Self {
        let mut bindings_msd = super::MsdForAllRates {
            msds: [Ssr {
                overall: 0.0,
                stream: 0.0,
                jumpstream: 0.0,
                handstream: 0.0,
                stamina: 0.0,
                jackspeed: 0.0,
                chordjack: 0.0,
                technical: 0.0,
            }; 14],
        };
        
        for (i, scores) in msd.msds.iter().enumerate() {
            bindings_msd.msds[i] = (*scores).into();
        }
        
        bindings_msd
    }
}

impl From<BindingsMsdForAllRates> for MsdForAllRates {
    fn from(bindings_msd: BindingsMsdForAllRates) -> Self {
        let mut msds = [SkillsetScores {
            overall: 0.0,
            stream: 0.0,
            jumpstream: 0.0,
            handstream: 0.0,
            stamina: 0.0,
            jackspeed: 0.0,
            chordjack: 0.0,
            technical: 0.0,
        }; 14];
        
        for (i, ssr) in bindings_msd.msds.iter().enumerate() {
            msds[i] = (*ssr).into();
        }
        
        MsdForAllRates { msds }
    }
}

/// Gestionnaire principal pour les calculs de difficulté
pub struct Calc {
    handle: *mut CalcHandle,
}

impl Calc {
    /// Crée une nouvelle instance de calculateur
    pub fn new() -> Result<Self, &'static str> {
        let handle = unsafe { create_calc() };
        if handle.is_null() {
            return Err("Failed to create calculator");
        }
        Ok(Calc { handle })
    }
    
    /// Obtient la version du calculateur
    pub fn version() -> i32 {
        unsafe { calc_version() }
    }
    
    /// Calcule les scores MSD pour tous les taux de musique
    pub fn calc_msd(&self, notes: &[Note]) -> Result<MsdForAllRates, &'static str> {
        if notes.is_empty() {
            return Err("No notes provided");
        }
        
        // Convertir les notes en format C
        let note_infos: Vec<NoteInfo> = notes.iter().map(|&note| note.into()).collect();
        
        let result = unsafe {
            calc_msd(self.handle, note_infos.as_ptr(), note_infos.len() as u64)
        };
        
        Ok(result.into())
    }
    
    /// Calcule les scores SSR pour un taux de musique et un objectif de score spécifiques
    pub fn calc_ssr(
        &self,
        notes: &[Note],
        music_rate: f32,
        score_goal: f32,
    ) -> Result<SkillsetScores, &'static str> {
        if notes.is_empty() {
            return Err("No notes provided");
        }
        
        if music_rate <= 0.0 {
            return Err("Music rate must be positive");
        }
        
        if score_goal <= 0.0 || score_goal > 100.0 {
            return Err("Score goal must be between 0 and 100");
        }
        
        // Convertir les notes en format C
        let mut note_infos: Vec<NoteInfo> = notes.iter().map(|&note| note.into()).collect();
        
        let result = unsafe {
            calc_ssr(self.handle, note_infos.as_mut_ptr(), note_infos.len() as u64, music_rate, score_goal)
        };
        
        Ok(result.into())
    }
}

impl Drop for Calc {
    fn drop(&mut self) {
        unsafe {
            destroy_calc(self.handle);
        }
    }
}

impl Default for Calc {
    fn default() -> Self {
        Self::new().expect("Failed to create default calculator")
    }
}

// Tests unitaires
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calc_version() {
        let version = Calc::version();
        assert!(version > 0);
    }
    
    #[test]
    fn test_calc_creation() {
        let calc = Calc::new();
        assert!(calc.is_ok());
    }
    
    #[test]
    fn test_note_conversion() {
        let note = Note {
            notes: 4,
            row_time: 1.5,
        };
        
        let note_info: NoteInfo = note.into();
        let converted_note: Note = note_info.into();
        
        assert_eq!(note.notes, converted_note.notes);
        assert_eq!(note.row_time, converted_note.row_time);
    }
    
    #[test]
    fn test_skillset_scores_conversion() {
        let scores = SkillsetScores {
            overall: 10.5,
            stream: 8.2,
            jumpstream: 12.1,
            handstream: 9.3,
            stamina: 7.8,
            jackspeed: 11.4,
            chordjack: 6.9,
            technical: 13.2,
        };
        
        let ssr: Ssr = scores.into();
        let converted_scores: SkillsetScores = ssr.into();
        
        assert_eq!(scores.overall, converted_scores.overall);
        assert_eq!(scores.stream, converted_scores.stream);
        assert_eq!(scores.jumpstream, converted_scores.jumpstream);
        assert_eq!(scores.handstream, converted_scores.handstream);
        assert_eq!(scores.stamina, converted_scores.stamina);
        assert_eq!(scores.jackspeed, converted_scores.jackspeed);
        assert_eq!(scores.chordjack, converted_scores.chordjack);
        assert_eq!(scores.technical, converted_scores.technical);
    }
}
