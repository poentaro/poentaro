/// A metadata field from a feature's snake_case key and value: `{ font_size: 12 }`
/// sets a text style, anything else is the value as written.
pub(super) fn metadata_field(key: &str, value: &str) -> String {
    let key = key.replace('_', " ");
    if value.starts_with('{') {
        format!("{key} style : {}", value.replace('_', " "))
    } else {
        format!("{key} : {value}")
    }
}
