//! What an API key looks like in a request, and how often its use is
//! written down.
#![allow(clippy::disallowed_macros)]

use live_share_worker::api_keys::{
    agent_prompt, bearer_key, expires_at, is_expired, should_touch, shown_prefix, ApiKeySummary,
    StoredApiKey, KEY_LIFETIME_MILLIS, TOUCH_EVERY_MILLIS,
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
fn a_key_works_for_thirty_days_after_it_is_created() {
    assert_eq!(KEY_LIFETIME_MILLIS, 30 * 24 * 60 * 60 * 1000);
    assert_eq!(expires_at(1_000), 1_000 + KEY_LIFETIME_MILLIS);
    assert!(!is_expired(1_000, 1_000 + KEY_LIFETIME_MILLIS - 1));
    assert!(is_expired(1_000, 1_000 + KEY_LIFETIME_MILLIS));
}

#[test]
fn a_listed_key_says_when_it_expires_and_whether_it_has() {
    let key = StoredApiKey {
        id: "k".to_string(),
        prefix: "poe_AbCd1234".to_string(),
        created_at: 1_000,
        last_used_at: None,
    };
    let fresh = ApiKeySummary::at(key.clone(), 2_000);
    assert_eq!(fresh.expires_at, 1_000 + KEY_LIFETIME_MILLIS);
    assert!(!fresh.expired);
    assert!(ApiKeySummary::at(key, 1_000 + KEY_LIFETIME_MILLIS).expired);
}

#[test]
fn the_prompt_stores_the_key_and_installs_a_skill_that_starts_with_the_schema() {
    let prompt = agent_prompt("https://worker.example", "poe_abc");
    assert!(prompt.contains("(umask 077 && printf '%s' 'poe_abc' > ~/.config/poentaro/key)"));
    assert!(prompt.contains("~/.agents/skills/poentaro/SKILL.md"));
    assert!(prompt.contains("ln -sfn ~/.agents/skills/poentaro ~/.claude/skills/poentaro"));
    assert!(prompt.contains("curl -s https://worker.example/agent \\\n"));
    assert!(prompt.contains("Start each session by sending `SCHEMA`."));
    assert!(!prompt.contains("{origin}") && !prompt.contains("{key}"));
    let skill: Vec<&str> = prompt.split("---8<---\n").collect();
    assert_eq!(skill.len(), 3, "the skill sits between two 8< lines");
    assert!(skill[1].starts_with("---\nname: poentaro\n"));
    assert!(
        !skill[1].contains("poe_abc"),
        "the skill reads the key from its file"
    );
}
