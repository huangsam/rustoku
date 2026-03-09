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

        /// Apply the W-Wing technique.
        const W_WING = 1 << 24;
        /// Apply the XY-Wing technique.
        const XY_WING = 1 << 25;
        /// Apply the XYZ-Wing technique.
        const XYZ_WING = 1 << 26;

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
