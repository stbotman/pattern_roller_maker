use super::*;

#[test]
fn test_lls_sse_exact_linear() {
    assert_eq!(lls_sse(1_f64, 2_f64, 3_f64, 4_f64), 0_f64);
    assert_eq!(lls_sse(2_f64, 4_f64, 6_f64, 8_f64), 0_f64);
    assert_eq!(lls_sse(1_f64, 1_f64, 1_f64, 1_f64), 0_f64);
}

#[test]
fn test_lls_sse_compare() {
    assert!(lls_sse(1_f64, 2_f64, 3_f64, 5.0) < lls_sse(1_f64, 2_f64, 3_f64, 5.1))
}

#[test]
fn test_cross_product_orts() {
    let a = Vector3::new(1_f64, 0_f64, 0_f64);
    let b = Vector3::new(0_f64, 1_f64, 0_f64);
    let c = Vector3::new(0_f64, 0_f64, 1_f64);
    assert_eq!(cross_product(a, b), c);
}

#[test]
fn test_cross_product_collinear() {
    let a = Vector3::new(1_f64, 2_f64, 3_f64);
    let b = Vector3::new(2_f64, 4_f64, 6_f64);
    let c = Vector3::new(0_f64, 0_f64, 0_f64);
    assert_eq!(cross_product(a, b), c);
}

#[test]
fn test_cross_product() {
    let a = Vector3::new(0.2, 0.3, 0.4);
    let b = Vector3::new(0.5, 0.6, 0.7);
    let c = Vector3::new(-0.03, 0.06, -0.03);
    assert_eq!(cross_product(a, b), c);
}

#[test]
fn test_vector_normalize() {
    let a = Vector3::new(2_f64, 10_f64, 11_f64);
    let b = Vector3::new(2.0 / 15.0, 2.0 / 3.0, 11.0 / 15.0);
    assert_eq!(a.normalize(), b);
}

#[test]
fn test_xy_perp_clockwise_orts() {
    let a = Vector3::new(1.0, 0.0, 0.0);
    let b = Vector3::new(0.0, 1.0, 0.0);
    assert_eq!(a.xy_perp_clockwise(), b);
}

#[test]
fn test_right_hand_orts() {
    let a = Vector3::new(1_f64, 0_f64, 0_f64);
    let b = Vector3::new(0_f64, 1_f64, 0_f64);
    let c = Vector3::new(0_f64, 0_f64, 1_f64);
    assert!(check_right_hand(&a, &b, &c));
    let a = Vector3::new(-1_f64, 0_f64, 0_f64);
    let b = Vector3::new(0_f64, -1_f64, 0_f64);
    let c = Vector3::new(0_f64, 0_f64, -1_f64);
    assert!(!check_right_hand(&a, &b, &c));
}
