use super::{singing_plan, Voice};

fn plan_of(parts: &str) -> super::SingingPlan {
    let source = format!(
        "parts [{parts}],\nscore \"\"\"\ntime=2/4 key=C4 bpm=60\n[M] 1 2\n[S] 我 们\n[A] 有 约\n\"\"\",\n"
    );
    let score = crate::compile(&source, "test.poentaro", &[]).expect("compiles");
    singing_plan(&score, 44_100).expect("plan")
}

#[test]
fn each_voiced_lyric_part_sings_in_its_voice_at_its_volume() {
    let plan = plan_of(
        r#""Melody" [M]: notes, "Soprano" [S]: lyrics [M] { voice: female, volume: 60% }, "Alto" [A]: lyrics [M] { voice: male }"#,
    );
    assert_eq!(plan.sample_rate, 44_100);
    assert_eq!(
        plan.parts
            .iter()
            .map(|part| (part.voice, part.volume_percent, part.notes.len()))
            .collect::<Vec<_>>(),
        vec![(Voice::Female, 60, 2), (Voice::Male, 100, 2)]
    );
}

#[test]
fn lyric_parts_without_a_voice_sing_nothing() {
    let plan = plan_of(r#""Melody" [M]: notes, "Soprano" [S]: lyrics [M], "Alto" [A]: lyrics [M]"#);
    assert!(plan.parts.is_empty());
}
