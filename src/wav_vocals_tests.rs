use super::mix_vocals;

#[test]
fn vocals_are_scaled_by_the_instruments_peak_and_added_to_both_channels() {
    let mut left = vec![0.5, -0.25];
    let mut right = vec![0.0, 0.0];
    mix_vocals(&mut left, &mut right, &[1.0, 0.5]);
    assert_eq!(left, vec![1.0, 0.0]);
    assert_eq!(right, vec![0.5, 0.25]);
}

#[test]
fn vocals_that_outlast_the_instruments_lengthen_the_mix() {
    let mut left = vec![0.5];
    let mut right = vec![0.5];
    mix_vocals(&mut left, &mut right, &[0.0, 1.0]);
    assert_eq!(left, vec![0.5, 0.5]);
    assert_eq!(right, vec![0.5, 0.5]);
}

#[test]
fn no_vocals_leave_the_instruments_alone() {
    let mut left = vec![0.5];
    let mut right = vec![-0.5];
    mix_vocals(&mut left, &mut right, &[]);
    assert_eq!(left, vec![0.5]);
    assert_eq!(right, vec![-0.5]);
}
