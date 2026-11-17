#![allow(clippy::disallowed_macros)]

use super::instant;
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn database_instants_preserve_milliseconds_on_both_sides_of_the_epoch() {
    assert_eq!(instant(0), UNIX_EPOCH);
    assert_eq!(instant(1234), UNIX_EPOCH + Duration::from_millis(1234));
    assert_eq!(instant(-1234), UNIX_EPOCH - Duration::from_millis(1234));
}
