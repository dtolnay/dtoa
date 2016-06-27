extern crate dtoa;

#[test]
fn test() {
    test_write(2.71828f64, "2.71828");
    test_write(0.0f64, "0.0");
    test_write(-0.0f64, "-0.0");
    test_write(1.1e128f64, "1.1e128");
    test_write(1.1e-64f64, "1.1e-64");
    test_write(2.718281828459045, "2.718281828459045");
    test_write(::std::f64::MAX, "1.7976931348623157e308");
}

fn test_write(value: f64, expected: &'static str) {
    let mut buf = Vec::with_capacity(30);
    dtoa::write(&mut buf, value).unwrap();
    let result = String::from_utf8(buf).unwrap();
    assert_eq!(result, expected.to_string());
}
