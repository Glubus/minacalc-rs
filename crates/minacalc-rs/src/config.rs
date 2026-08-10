use crate::Error;

/// Final multipliers applied to each modeled skillset before Overall is aggregated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkillsetScalers {
    pub stream: f32,
    pub jumpstream: f32,
    pub handstream: f32,
    pub stamina: f32,
    pub jackspeed: f32,
    pub chordjack: f32,
    pub technical: f32,
}

impl Default for SkillsetScalers {
    fn default() -> Self {
        Self {
            stream: 1.0,
            jumpstream: 1.0,
            handstream: 1.0,
            stamina: 1.0,
            jackspeed: 1.0,
            chordjack: 1.0,
            technical: 1.0,
        }
    }
}

impl SkillsetScalers {
    pub(crate) fn values(self) -> [f32; 7] {
        [
            self.stream,
            self.jumpstream,
            self.handstream,
            self.stamina,
            self.jackspeed,
            self.chordjack,
            self.technical,
        ]
    }
}

/// Complete per-calculator configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalcConfig {
    pub ssr_goal_cap: f32,
    pub low_acc_cutoff: f32,
    /// `None` disables the per-skillset SSR cap.
    pub ssr_rating_cap: Option<f32>,
    pub default_score_goal: f32,
    pub grind_scaling: bool,
    pub skillset_scalers: SkillsetScalers,
}

impl Default for CalcConfig {
    fn default() -> Self {
        Self {
            ssr_goal_cap: 0.965,
            low_acc_cutoff: 0.9,
            ssr_rating_cap: Some(40.0),
            default_score_goal: 0.93,
            grind_scaling: true,
            skillset_scalers: SkillsetScalers::default(),
        }
    }
}

impl CalcConfig {
    /// Validate all values before they cross the native boundary.
    pub fn validate(&self) -> Result<(), Error> {
        validate_goal("ssr_goal_cap", self.ssr_goal_cap)?;
        validate_goal("low_acc_cutoff", self.low_acc_cutoff)?;
        validate_goal("default_score_goal", self.default_score_goal)?;
        if self
            .ssr_rating_cap
            .is_some_and(|value| !value.is_finite() || value <= 0.0)
        {
            return Err(Error::InvalidConfig(
                "ssr_rating_cap must be finite and greater than 0",
            ));
        }
        if self
            .skillset_scalers
            .values()
            .into_iter()
            .any(|value| !value.is_finite() || value <= 0.0)
        {
            return Err(Error::InvalidConfig(
                "skillset scalers must be finite and greater than 0",
            ));
        }
        Ok(())
    }
}

fn validate_goal(name: &'static str, value: f32) -> Result<(), Error> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(Error::InvalidConfig(match name {
            "ssr_goal_cap" => "ssr_goal_cap must be finite and between 0 and 1",
            "low_acc_cutoff" => "low_acc_cutoff must be finite and between 0 and 1",
            _ => "default_score_goal must be finite and between 0 and 1",
        }))
    }
}
