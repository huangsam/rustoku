use bitflags::bitflags;

bitflags! {
    /// Bitflags indicating which human techniques are active/enabled.
    ///
    /// The flags are organized into bytes for better organization:
    /// - Easy techniques from bits 0-7
    /// - Medium techniques from bits 8-15
    /// - Hard techniques from bits 16-23
    /// - Expert techniques from bits 24-31
    ///
    /// Composite groups (EASY, MEDIUM, HARD, EXPERT) are here for convenience.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TechniqueFlags: u32 {
        /// Apply the naked singles technique.
        const NAKED_SINGLES = 1 << 0;
        /// Apply the hidden singles technique.
        const HIDDEN_SINGLES = 1 << 1;

        /// Apply the naked pairs technique.
        const NAKED_PAIRS = 1 << 8;
        /// Apply the hidden pairs technique.
        const HIDDEN_PAIRS = 1 << 9;
        /// Apply the locked candidates technique.
        const LOCKED_CANDIDATES = 1 << 10;
        /// Apply the naked triples technique.
        const NAKED_TRIPLES = 1 << 11;
        /// Apply the hidden triples technique.
        const HIDDEN_TRIPLES = 1 << 12;

        /// Apply the X-Wing technique.
        const X_WING = 1 << 16;
        /// Apply the naked quads technique.
        const NAKED_QUADS = 1 << 17;
        /// Apply the hidden quads technique.
        const HIDDEN_QUADS = 1 << 18;
        /// Apply the Swordfish technique.
        const SWORDFISH = 1 << 19;
        /// Apply the Jellyfish technique.
        const JELLYFISH = 1 << 20;
        /// Apply the Skyscraper technique.
        const SKYSCRAPER = 1 << 21;

        /// Apply the W-Wing technique.
        const W_WING = 1 << 24;
        /// Apply the XY-Wing technique.
        const XY_WING = 1 << 25;
        /// Apply the XYZ-Wing technique.
        const XYZ_WING = 1 << 26;
        /// Apply the Alternating Inference Chain (AIC) technique.
        const ALTERNATING_INFERENCE_CHAIN = 1 << 27;

        /// Alias for X_WING.
        #[deprecated(note = "use X_WING instead")]
        const XWING = Self::X_WING.bits();

        /// Apply easy techniques
        const EASY = 0x0000_00FF;
        /// Apply medium techniques
        const MEDIUM = 0x0000_FF00;
        /// Apply hard techniques
        const HARD = 0x00FF_0000;
        /// Apply expert techniques
        const EXPERT = 0xFF00_0000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.difficulty_name())
    }
}

impl From<TechniqueFlags> for Difficulty {
    fn from(flags: TechniqueFlags) -> Self {
        if !(flags & TechniqueFlags::EXPERT).is_empty() {
            Difficulty::Expert
        } else if !(flags & TechniqueFlags::HARD).is_empty() {
            Difficulty::Hard
        } else if !(flags & TechniqueFlags::MEDIUM).is_empty() {
            Difficulty::Medium
        } else {
            Difficulty::Easy
        }
    }
}

impl TechniqueFlags {
    /// Returns the highest difficulty level associated with the current flags.
    pub fn difficulty(&self) -> Difficulty {
        Difficulty::from(*self)
    }

    /// Returns the name of the highest difficulty level.
    pub fn difficulty_name(&self) -> &'static str {
        self.difficulty().difficulty_name()
    }
}

impl Difficulty {
    pub fn difficulty_name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Expert => "Expert",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_display() {
        assert_eq!(format!("{}", Difficulty::Easy), "Easy");
        assert_eq!(format!("{}", Difficulty::Medium), "Medium");
        assert_eq!(format!("{}", Difficulty::Hard), "Hard");
        assert_eq!(format!("{}", Difficulty::Expert), "Expert");
    }

    #[test]
    fn test_difficulty_name() {
        assert_eq!(Difficulty::Easy.difficulty_name(), "Easy");
        assert_eq!(Difficulty::Medium.difficulty_name(), "Medium");
        assert_eq!(Difficulty::Hard.difficulty_name(), "Hard");
        assert_eq!(Difficulty::Expert.difficulty_name(), "Expert");
    }

    #[test]
    fn test_technique_flags_difficulty() {
        // Single flags
        assert_eq!(TechniqueFlags::NAKED_SINGLES.difficulty(), Difficulty::Easy);
        assert_eq!(TechniqueFlags::NAKED_PAIRS.difficulty(), Difficulty::Medium);
        assert_eq!(TechniqueFlags::X_WING.difficulty(), Difficulty::Hard);
        assert_eq!(TechniqueFlags::W_WING.difficulty(), Difficulty::Expert);

        // Combined flags (should return highest difficulty)
        let combined = TechniqueFlags::NAKED_SINGLES | TechniqueFlags::X_WING;
        assert_eq!(combined.difficulty(), Difficulty::Hard);

        let all = TechniqueFlags::all();
        assert_eq!(all.difficulty(), Difficulty::Expert);
    }

    #[test]
    fn test_from_technique_flags_for_difficulty() {
        assert_eq!(
            Difficulty::from(TechniqueFlags::HIDDEN_SINGLES),
            Difficulty::Easy
        );
        assert_eq!(
            Difficulty::from(TechniqueFlags::LOCKED_CANDIDATES),
            Difficulty::Medium
        );
        assert_eq!(
            Difficulty::from(TechniqueFlags::JELLYFISH),
            Difficulty::Hard
        );
        assert_eq!(
            Difficulty::from(TechniqueFlags::XYZ_WING),
            Difficulty::Expert
        );
    }
}
