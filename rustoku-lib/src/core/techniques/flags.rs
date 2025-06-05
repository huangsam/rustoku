use bitflags::bitflags;

bitflags! {
    /// Bitmask to control which human techniques are applied.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TechniqueFlags: u16 {
        /// Apply the naked singles technique.
        const NAKED_SINGLES = 1 << 0;
        /// Apply the hidden singles technique.
        const HIDDEN_SINGLES = 1 << 1;
        /// Apply the naked pairs technique.
        const NAKED_PAIRS = 1 << 2;
        /// Apply the hidden pairs technique.
        const HIDDEN_PAIRS = 1 << 3;
        /// Apply the locked candidates technique.
        const LOCKED_CANDIDATES = 1 << 4;
        /// Apply the X-Wing technique.
        const XWING = 1 << 5;

        /// Apply easy techniques like naked singles.
        const EASY = Self::NAKED_SINGLES.bits() | Self::HIDDEN_SINGLES.bits();
        /// Apply medium techniques like naked pairs.
        const MEDIUM = Self::NAKED_PAIRS.bits() | Self::HIDDEN_PAIRS.bits() | Self::LOCKED_CANDIDATES.bits();
        /// Apply hard techniques like X-Wings.
        const HARD = Self::XWING.bits();
    }
}
