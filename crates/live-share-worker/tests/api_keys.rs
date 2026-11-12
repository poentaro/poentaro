//! What an API key looks like in a request, and how often its use is
//! written down.
#![allow(clippy::disallowed_macros)]

use live_share_worker::api_keys::{
    agent_prompt, bearer_key, should_touch, shown_prefix, TOUCH_EVERY_MILLIS,
};

#[test]
fn a_bearer_header_gives_its_key() {
    assert_eq!(bearer_key(Some("Bearer poe_abc")), Some("poe_abc"));
    assert_eq!(bearer_key(Some("  Bearer   poe_abc  ")), Some("poe_abc"));
}

#[test]
fn anything_but_a_bearer_poentaro_key_gives_none() {
    assert_eq!(bearer_key(None), None);
    assert_eq!(bearer_key(Some("poe_abc")), None);
    assert_eq!(bearer_key(Some("Basic poe_abc")), None);
    assert_eq!(bearer_key(Some("Bearer sk-abc")), None);
}

#[test]
fn a_key_is_named_by_its_first_twelve_characters() {
    assert_eq!(shown_prefix("poe_AbCd1234EfGh5678"), "poe_AbCd1234");
    assert_eq!(shown_prefix("poe_ab"), "poe_ab");
}

#[test]
fn a_use_is_written_down_at_most_once_a_minute() {
    assert!(should_touch(None, 0));
    assert!(!should_touch(Some(1_000), 1_000 + TOUCH_EVERY_MILLIS - 1));
    assert!(should_touch(Some(1_000), 1_000 + TOUCH_EVERY_MILLIS));
}

#[test]
fn the_prompt_sends_the_agent_to_the_guide_with_its_key() {
    assert_eq!(
        agent_prompt("https://worker.example", "poe_abc"),
        "Help me with my music scores on Poentaro. Before anything else, run\n\
         curl -s https://worker.example/agent -H \"Authorization: Bearer poe_abc\"\n\
         and follow what it says.\n"
    );
}
