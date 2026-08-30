use rstest::{fixture, rstest};
use wasserxr::utils::rotation_buffer::{RotationBuffer, VecRotationBuffer};

#[fixture]
fn rotator() -> VecRotationBuffer<usize> {
    VecRotationBuffer::new()
}

#[rstest]
fn is_empty_reflects_readable_values(mut rotator: VecRotationBuffer<usize>) {
    assert!(rotator.is_empty());

    rotator.push(1);
    assert!(rotator.is_empty());

    rotator.rotate();
    assert!(!rotator.is_empty());

    rotator.clear();
    assert!(rotator.is_empty());
}

#[rstest]
#[case(&[])]
#[case(&[1])]
#[case(&[1,2,3])]
fn test_lifecycle(mut rotator: VecRotationBuffer<usize>, #[case] input: &[usize]) {
    assert_eq!(rotator.len(), 0);
    assert_eq!(rotator.as_slice(), &[]);
    assert_eq!(rotator.get(0), None);

    // Have one half be filled before rotation and the other half after rotation
    let (left, right) = input.split_at(input.len() / 2);

    left.iter().for_each(|x| rotator.push(*x));
    assert_eq!(rotator.len(), 0);

    rotator.rotate();

    right.iter().for_each(|x| rotator.push(*x));

    assert_eq!(rotator.len(), left.len());
    assert_eq!(rotator.as_slice(), left);
    assert!(rotator.iter().eq(left.iter()));
    if !left.is_empty() {
        assert_eq!(*rotator.get(0).unwrap(), left[0]);
    }
    rotator.rotate();
    assert_eq!(rotator.len(), right.len());
    assert_eq!(rotator.as_slice(), right);
    assert!(rotator.iter().eq(right.iter()));
    if !left.is_empty() {
        assert_eq!(*rotator.get(0).unwrap(), right[0]);
    }
    rotator.rotate();
    assert_eq!(rotator.len(), 0);
    assert_eq!(rotator.as_slice(), &[]);
    assert_eq!(rotator.get(0), None);
}

#[rstest]
#[case(&[])]
#[case(&[1])]
#[case(&[1,2,3])]
fn test_clear(mut rotator: VecRotationBuffer<usize>, #[case] input: &[usize]) {
    input.iter().for_each(|x| rotator.push(*x));

    rotator.clear();
    rotator.rotate();

    assert_eq!(rotator.as_slice(), &[]);
}
