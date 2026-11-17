/// Declares a fieldless enum whose variants each map to one `.poentaro` source
/// keyword, generating `ALL` (declaration order), `keyword()`, and
/// `from_keyword()` from a single `Variant => "keyword"` table — so the
/// spelling of every keyword lives in exactly one place, and the parser, the
/// source writer, and the wasm boundary all dispatch on the enum instead of
/// re-matching string literals.
macro_rules! keyword_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$variant_meta:meta])* $variant:ident => $keyword:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $($(#[$variant_meta])* $variant),+
        }

        impl $name {
            /// Every variant, in declaration order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// This variant's `.poentaro` source spelling.
            pub fn keyword(self) -> &'static str {
                match self {
                    $(Self::$variant => $keyword),+
                }
            }

            /// The variant spelled `keyword` in `.poentaro` source, if any.
            pub fn from_keyword(keyword: &str) -> Option<Self> {
                match keyword {
                    $($keyword => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}
