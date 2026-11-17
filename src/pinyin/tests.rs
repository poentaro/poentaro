use super::PinyinSyllable;

fn marked(text: &str) -> Option<String> {
    PinyinSyllable::parse(text).map(|syllable| syllable.marked())
}

fn numbered(text: &str) -> Option<String> {
    PinyinSyllable::parse(text).map(|syllable| syllable.numbered())
}

#[test]
fn tone_marks_and_tone_numbers_name_the_same_reading() {
    assert_eq!(
        PinyinSyllable::parse("háng"),
        PinyinSyllable::parse("hang2")
    );
    assert_eq!(marked("hang2").as_deref(), Some("háng"));
    assert_eq!(numbered("háng").as_deref(), Some("hang2"));
}

#[test]
fn a_syllable_without_a_tone_is_neutral() {
    assert_eq!(marked("hang").as_deref(), Some("hang"));
    assert_eq!(numbered("hang").as_deref(), Some("hang5"));
    assert_eq!(marked("de5").as_deref(), Some("de"));
}

#[test]
fn the_mark_goes_on_a_or_e_then_the_o_of_ou_then_the_last_vowel() {
    assert_eq!(marked("zhang3").as_deref(), Some("zhǎng"));
    assert_eq!(marked("xie4").as_deref(), Some("xiè"));
    assert_eq!(marked("dou1").as_deref(), Some("dōu"));
    assert_eq!(marked("gui4").as_deref(), Some("guì"));
    assert_eq!(marked("liu2").as_deref(), Some("liú"));
    assert_eq!(marked("huo3").as_deref(), Some("huǒ"));
}

#[test]
fn u_umlaut_may_be_written_v() {
    assert_eq!(marked("lv4").as_deref(), Some("lǜ"));
    assert_eq!(marked("lü4").as_deref(), Some("lǜ"));
    assert_eq!(numbered("lǜ").as_deref(), Some("lv4"));
    assert_eq!(marked("nve4").as_deref(), Some("nüè"));
}

#[test]
fn capitals_read_as_lowercase() {
    assert_eq!(marked("HANG2").as_deref(), Some("háng"));
}

#[test]
fn text_that_is_not_one_syllable_is_rejected() {
    for text in [
        "", "hello", "hangzhou", "háng2", "hǎó", "hang6", "hang0", "2", "陪", "ha ng",
    ] {
        assert_eq!(PinyinSyllable::parse(text), None, "{text:?}");
    }
}
