use bitflags::bitflags;

bitflags! {
    /// Bitflags indicating which human techniques are active/enabled.
    ///
    /// The flags are organized into bytes for better organization:
    /// - Byte 0: Singles & Subsets (0-7)
    /// - Byte 1: Intersections (8-15)
    /// - Byte 2: Fish (16-23)
    /// - Byte 3: Wings & Chains & Other (24-31)
    ///
    /// Composite groups (EASY, MEDIUM, HARD) are also provided for convenience.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TechniqueFlags: u32 {
        /// Apply the naked singles technique.
        const NAKED_SINGLES = 1 << 0;
        /// Apply the hidden singles technique.
        const HIDDEN_SINGLES = 1 << 1;

        /// Apply the naked pairs technique.
        const NAKED_PAIRS = 1 << 4;
        /// Apply the hidden pairs technique.
        const HIDDEN_PAIRS = 1 << 5;
        /// Apply the naked triples technique.
        const NAKED_TRIPLES = 1 << 6;
        /// Apply the hidden triples technique.
        const HIDDEN_TRIPLES = 1 << 7;

        /// Apply the locked candidates technique.
        const LOCKED_CANDIDATES = 1 << 8;

        /// Apply the X-Wing technique.
        const X_WING = 1 << 16;
        /// Apply the Swordfish technique.
        const SWORDFISH = 1 << 17;

        /// Apply the XY-Wing technique.
        const XY_WING = 1 << 24;
        /// Apply the XYZ-Wing technique.
        const XYZ_WING = 1 << 25;
        /// Apply the W-Wing technique.
        const W_WING = 1 << 26;

        /// Alias for X_WING.
        #[deprecated(note = "use X_WING instead")]
        const XWING = Self::X_WING.bits();

        /// Apply easy techniques like naked singles.
        const EASY = Self::NAKED_SINGLES.bits() | Self::HIDDEN_SINGLES.bits();
        /// Apply medium techniques like naked pairs.
        const MEDIUM = Self::NAKED_PAIRS.bits()
            | Self::HIDDEN_PAIRS.bits()
            | Self::LOCKED_CANDIDATES.bits()
            | Self::NAKED_TRIPLES.bits()
            | Self::HIDDEN_TRIPLES.bits();
        /// Apply hard techniques like X-Wings.
        const HARD = Self::X_WING.bits()
            | Self::SWORDFISH.bits()
            | Self::XY_WING.bits()
            | Self::XYZ_WING.bits()
            | Self::W_WING.bits();
    }
}
