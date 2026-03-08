use bitflags::bitflags;

bitflags! {
    /// Bitflags indicating which human techniques are active/enabled.
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
        /// Apply the Swordfish technique.
        const SWORDFISH = 1 << 6;
        /// Apply the XY-Wing technique.
        const XY_WING = 1 << 7;
        /// Apply the naked triples technique.
        const NAKED_TRIPLES = 1 << 8;
        /// Apply the hidden triples technique.
        const HIDDEN_TRIPLES = 1 << 9;
        /// Apply the W-Wing technique.
        const W_WING = 1 << 10;

        /// Apply easy techniques like naked singles.
        const EASY = Self::NAKED_SINGLES.bits() | Self::HIDDEN_SINGLES.bits();
        /// Apply medium techniques like naked pairs.
        const MEDIUM = Self::NAKED_PAIRS.bits() | Self::HIDDEN_PAIRS.bits() | Self::LOCKED_CANDIDATES.bits() | Self::NAKED_TRIPLES.bits() | Self::HIDDEN_TRIPLES.bits();
        /// Apply hard techniques like X-Wings.
        const HARD = Self::XWING.bits() | Self::SWORDFISH.bits() | Self::XY_WING.bits() | Self::W_WING.bits();
    }
}
