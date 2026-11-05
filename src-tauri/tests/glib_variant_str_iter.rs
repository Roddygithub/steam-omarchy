use glib::variant::ToVariant;

// Run with --release: the immutable C out-pointer was miscompiled with optimization.
#[test]
fn variant_str_iter_out_pointer() {
    let variant = ["zero", "one", "two", "three", "four", "five"].to_variant();
    let mut iter = variant.array_iter_str().unwrap();
    assert_eq!(iter.next(), Some("zero"));
    assert_eq!(iter.next_back(), Some("five"));
    assert_eq!(iter.nth(1), Some("two"));
    assert_eq!(iter.nth_back(1), Some("three"));
    assert_eq!(iter.next(), None);
    assert_eq!(variant.array_iter_str().unwrap().last(), Some("five"));
    let empty = Vec::<String>::new().to_variant();
    assert_eq!(empty.array_iter_str().unwrap().next(), None);
}
