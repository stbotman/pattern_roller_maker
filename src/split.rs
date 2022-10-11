use crate::parameters::Parameters;

pub fn split_quad_optimal(params: &Parameters, i: usize, j: usize) -> (bool, f64, f64, f64, f64) {
    let (i, j) = (i as i32, j as i32);
    let corner_tl = params.get_rho_looped(i - 1, j - 1);
    let corner_tr = params.get_rho_looped(i - 1, j + 2);
    let corner_bl = params.get_rho_looped(i + 2, j - 1);
    let corner_br = params.get_rho_looped(i + 2, j + 2);
    let quad_tl = params.get_rho_looped(i, j);
    let quad_tr = params.get_rho_looped(i + 1, j);
    let quad_bl = params.get_rho_looped(i, j + 1);
    let quad_br = params.get_rho_looped(i + 1, j + 1);
    let tlbr_split_score = lls_sse(corner_tl, quad_tl, quad_br, corner_br);
    let trbl_split_score = lls_sse(corner_tr, quad_tr, quad_bl, corner_bl);
    let tlbr_split = tlbr_split_score < trbl_split_score;
    (tlbr_split, quad_tl, quad_tr, quad_bl, quad_br)
}

fn lls_sse(y1: f64, y2: f64, y3: f64, y4: f64) -> f64 {
    let y_sum = y1 + y2 + y3 + y4;
    let xy_sum = y2 + 2.0 * y3 + 3.0 * y4;
    let y_squared_sum = y1 * y1 + y2 * y2 + y3 * y3 + y4 * y4;
    let ss_yy = y_squared_sum - y_sum * y_sum * 0.25;
    let ss_xy = xy_sum - 1.5 * y_sum;
    let sse = ss_yy - ss_xy * ss_xy * 0.2;
    sse
}

#[cfg(test)]
#[test]
fn test_lls_sse_exact_linear() {
    assert_eq!(lls_sse(1.0, 2.0, 3.0, 4.0), 0.0);
    assert_eq!(lls_sse(2.0, 4.0, 6.0, 8.0), 0.0);
    assert_eq!(lls_sse(1.0, 1.0, 1.0, 1.0), 0.0);
}

#[test]
fn test_lls_sse_compare() {
    assert!(lls_sse(1.0, 2.0, 3.0, 5.0) < lls_sse(1.0, 2.0, 3.0, 5.1))
}
