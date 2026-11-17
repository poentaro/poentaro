//! The valid ranges of a `parts` line's numeric settings. The parser clamps
//! an out-of-range value into its range (reporting a diagnostic), so nothing
//! downstream of it — MIDI, audio, the Edit Parts modal — ever sees one.

/// An inclusive `min..=max` range of one part setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartSettingRange<T> {
    pub min: T,
    pub max: T,
}

/// A written setting value moved into its [`PartSettingRange`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClampedSetting<T> {
    pub value: T,
    /// Whether the written value lay outside the range.
    pub was_out_of_range: bool,
}

impl<T: Copy + Into<i32> + TryFrom<i32>> PartSettingRange<T> {
    /// `written` clamped into this range. Takes an `i32` so a value too wide
    /// for `T` itself (e.g. `300%` for a `u8` volume) still clamps.
    pub fn clamp(&self, written: i32) -> ClampedSetting<T> {
        let clamped = written.clamp(self.min.into(), self.max.into());
        ClampedSetting {
            value: T::try_from(clamped).unwrap_or(self.max),
            was_out_of_range: clamped != written,
        }
    }
}

/// The valid range of every numeric `parts` setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartSettingLimits {
    /// The `N%` suffix. `0%` silences a part; `100%` is full MIDI volume.
    pub volume: PartSettingRange<u8>,
    /// The `+N`/`-N` suffix, in octaves.
    pub octave_offset: PartSettingRange<i8>,
}

pub const PART_SETTING_LIMITS: PartSettingLimits = PartSettingLimits {
    volume: PartSettingRange { min: 0, max: 100 },
    octave_offset: PartSettingRange { min: -4, max: 4 },
};
