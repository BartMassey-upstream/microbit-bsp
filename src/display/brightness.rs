/// A brightness setting for the display.
#[derive(Clone, Copy)]
pub struct Brightness<const LEVELS: u8>(u8);

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Indication of a brightness error issue.
pub enum BrightnessError {
    /// Requested brightness above maximum.
    TooBright,
}

impl<const LEVELS: u8> Brightness<LEVELS> {
    /// Maximum brightness
    pub const MAX: Brightness<LEVELS> = Brightness(LEVELS - 1);

    /// Lowest brightness
    pub const MIN: Brightness<LEVELS> = Brightness(0);

    /// Create a new brightness with a custom level.
    /// Fails if requested level is too large.
    pub fn new(level: u8) -> Result<Self, BrightnessError> {
        if level >= LEVELS {
            return Err(BrightnessError::TooBright);
        }
        Ok(Self(level))
    }

    /// Return the level value
    pub fn level(&self) -> u8 {
        self.0
    }
}

impl<const LEVELS: u8> Default for Brightness<LEVELS> {
    fn default() -> Self {
        Self::MAX
    }
}

impl<const BLEVELS: u8> core::ops::AddAssign<u8> for Brightness<BLEVELS> {
    fn add_assign(&mut self, rhs: u8) {
        self.0 += core::cmp::min(Self::MAX.level() - self.0, rhs);
    }
}

impl<const BLEVELS: u8> core::ops::SubAssign<u8> for Brightness<BLEVELS> {
    fn sub_assign(&mut self, rhs: u8) {
        self.0 -= core::cmp::min(self.0, rhs);
    }
}
