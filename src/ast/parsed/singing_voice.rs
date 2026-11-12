//! The voice a lyric part sings its lyrics in, in exported audio
//! (`lyrics [S] { voice: female }`; see `syntax.md`, "Singing").

keyword_enum! {
    /// The body of a synthesized singer. A lyric part without one is silent.
    pub enum SingingVoice {
        Male => "male",
        Female => "female",
        Child => "child",
    }
}
