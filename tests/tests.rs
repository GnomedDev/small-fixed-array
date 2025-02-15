use small_fixed_array::FixedArray;

#[test]
fn check_zst_functionality() {
    let array = FixedArray::<(), u32>::from_vec_trunc(vec![(); 16]);
    assert!(!array.is_empty());
    assert_eq!(array.len(), 16);
}
