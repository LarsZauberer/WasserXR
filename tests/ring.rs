use rstest::{fixture, rstest};
use wasserxr::utils::ring::Ring;

#[fixture]
fn basic() -> Ring<usize> {
    Ring::new(2)
}

#[rstest]
fn is_empty_reflects_length(mut basic: Ring<usize>) {
    assert!(basic.is_empty());

    basic.push(1);
    assert!(!basic.is_empty());

    basic.clear();
    assert!(basic.is_empty());
}

#[rstest]
fn shrinking_capacity_drops_only_excess_values() {
    let mut ring = Ring::new(4);
    ring.push(1);
    ring.push(2);

    ring.set_capacity(3);
    assert_eq!(ring.iter().copied().collect::<Vec<_>>(), vec![1, 2]);

    ring.set_capacity(1);
    assert_eq!(ring.iter().copied().collect::<Vec<_>>(), vec![2]);
}

#[rstest]
#[case(&[], &[], 0)]
#[case(&[1], &[1], 1)]
#[case(&[1,2], &[1,2], 2)]
#[case(&[1,2,3], &[2,3], 2)]
#[case(&[1,2,3,4], &[3,4], 2)]
fn simple_cycle(
    mut basic: Ring<usize>,
    #[case] input: &[usize],
    #[case] expected: &[usize],
    #[case] expected_length: usize,
) {
    // Assert that adding and outputing is possible
    input.iter().for_each(|x| basic.push(*x));
    let output: Vec<usize> = basic.iter().cloned().collect();
    assert_eq!(&output, expected);

    // Assert capacity
    assert_eq!(basic.cap(), 2);

    // Assert length
    assert_eq!(basic.len(), expected_length)
}

#[rstest]
fn simple_clear(mut basic: Ring<usize>) {
    basic.push(1);
    basic.clear();

    // Assert basic properties
    assert_eq!(basic.cap(), 2);
    assert_eq!(basic.len(), 0);

    // Assert that any output is also empty
    let output: Vec<usize> = basic.iter().cloned().collect();
    assert_eq!(&output, &[]);

    let output = basic.get(0);
    assert_eq!(output, None);
}

#[rstest]
#[case(&[])]
#[case(&[1])]
#[case(&[1,2])]
#[case(&[2,3])]
fn simple_mutate(mut basic: Ring<usize>, #[case] input: &[usize]) {
    // Add the input
    input.iter().for_each(|x| basic.push(*x));
    let mut oracle = input.to_vec();

    // Single modification
    if let Some(value) = oracle.first_mut() {
        *value += 1;
        *basic.get_mut(0).unwrap() += 1;
    }

    // Mutate the whole ring
    oracle.iter_mut().for_each(|x| *x += 1);
    basic.iter_mut().for_each(|x| *x += 1);
    let output: Vec<usize> = basic.iter().cloned().collect();
    assert_eq!(output, oracle);
}
