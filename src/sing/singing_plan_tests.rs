use super::{singing_plan, Voice};

fn plan_of(parts: &str) -> super::SingingPlan {
    let source = format!(
        "parts [{parts}],\nscore \"\"\"\ntime=2/4 key=C4 bpm=60\n[S] 1 2\n[A] 1 2\n[w] 我 们\n\"\"\",\n"
    );
    let score = crate::compile(&source, "test.poentaro", &[]).expect("compiles");
    singing_plan(&score, 44_100).expect("plan")
}

#[test]
fn each_singing_part_sings_in_its_voice_at_its_volume() {
    let plan = plan_of(
        r#""Soprano" [S]: notes { instrument: soprano, volume: 60% }, "Alto" [A]: notes { sound: chimera tenor }, "Words" [w]: lyrics [S, A]"#,
    );
    assert_eq!(plan.sample_rate, 44_100);
    assert_eq!(
        plan.parts
            .iter()
            .map(|part| (part.voice, part.volume_percent, part.notes.len()))
            .collect::<Vec<_>>(),
        vec![(Voice::Soprano, 60, 2), (Voice::Tenor, 100, 2)]
    );
}

#[test]
fn parts_without_a_chimera_sound_sing_nothing() {
    let plan = plan_of(r#""Soprano" [S]: notes, "Alto" [A]: notes, "Words" [w]: lyrics [S, A]"#);
    assert!(plan.parts.is_empty());
}
