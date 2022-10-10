use std::f64::EPSILON;
use std::fmt;

pub struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector3 {
    pub const UP: Vector3 = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    pub const DOWN: Vector3 = Vector3 {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };

    pub const ZERO: Vector3 = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 { x: x, y: y, z: z }
    }

    pub fn from_points(origin: &Vector3, end: &Vector3) -> Vector3 {
        Vector3 {
            x: end.x - origin.x,
            y: end.y - origin.y,
            z: end.z - origin.z,
        }
    }

    pub fn from_cross_product(vec_a: Vector3, vec_b: Vector3) -> Vector3 {
        let x = vec_a.y * vec_b.z - vec_a.z * vec_b.y;
        let y = vec_a.z * vec_b.x - vec_a.x * vec_b.z;
        let z = vec_a.x * vec_b.y - vec_a.y * vec_b.x;
        Vector3 { x: x, y: y, z: z }
    }

    pub fn normalize(mut self) -> Self {
        let scale: f64 = (self.x.powi(2) + self.y.powi(2) + self.z.powi(2))
            .sqrt()
            .recip();
        self.x = self.x * scale;
        self.y = self.y * scale;
        self.z = self.z * scale;
        self
    }

    pub fn xy_perp_clockwise(self) -> Vector3 {
        Vector3 {
            x: -self.y,
            y: self.x,
            z: self.z,
        }
    }

    pub fn to_binary(&self) -> [u8; 3 * 4] {
        let mut binv: [u8; 12] = [0; 12];
        binv[0..4].copy_from_slice(&({ self.x as f32 }.to_le_bytes()));
        binv[4..8].copy_from_slice(&({ self.y as f32 }.to_le_bytes()));
        binv[8..12].copy_from_slice(&({ self.z as f32 }.to_le_bytes()));
        binv
    }
}

impl fmt::Debug for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x:{:e}, y:{:e}. z:{:e})", self.x, self.y, self.z)
    }
}

impl PartialEq for Vector3 {
    fn eq(&self, other_vec: &Self) -> bool {
        let x_is_close = (self.x - other_vec.x).abs() <= EPSILON;
        let y_is_close = (self.y - other_vec.y).abs() <= EPSILON;
        let z_is_close = (self.z - other_vec.z).abs() <= EPSILON;
        x_is_close && y_is_close && z_is_close
    }
}

pub fn check_right_hand(vec_a: &Vector3, vec_b: &Vector3, vec_c: &Vector3) -> bool {
    let det: f64 =
        vec_a.x * vec_b.y * vec_c.z + vec_a.y * vec_b.z * vec_c.x + vec_a.z * vec_b.x * vec_c.y
            - vec_a.z * vec_b.y * vec_c.x
            - vec_a.y * vec_b.x * vec_c.z
            - vec_a.x * vec_b.z * vec_c.y;
    det > 0.0
}

pub fn xy_scalar_product(vec_a: &Vector3, vec_b: &Vector3) -> f64 {
    vec_a.x * vec_b.x + vec_a.y * vec_b.y
}

#[cfg(test)]
#[test]
fn test_vector_normalize() {
    let a = Vector3::new(2.0, 10.0, 11.0);
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
fn test_cross_product_orts() {
    let a = Vector3::new(1.0, 0.0, 0.0);
    let b = Vector3::new(0.0, 1.0, 0.0);
    let c = Vector3::new(0.0, 0.0, 1.0);
    assert_eq!(Vector3::from_cross_product(a, b), c);
}

#[test]
fn test_cross_product_collinear() {
    let a = Vector3::new(1.0, 2.0, 3.0);
    let b = Vector3::new(2.0, 4.0, 6.0);
    let c = Vector3::new(0.0, 0.0, 0.0);
    assert_eq!(Vector3::from_cross_product(a, b), c);
}

#[test]
fn test_cross_product() {
    let a = Vector3::new(0.2, 0.3, 0.4);
    let b = Vector3::new(0.5, 0.6, 0.7);
    let c = Vector3::new(-0.03, 0.06, -0.03);
    assert_eq!(Vector3::from_cross_product(a, b), c);
}

#[test]
fn test_right_hand_orts() {
    let a = Vector3::new(1.0, 0.0, 0.0);
    let b = Vector3::new(0.0, 1.0, 0.0);
    let c = Vector3::new(0.0, 0.0, 1.0);
    assert!(check_right_hand(&a, &b, &c));
    let a = Vector3::new(-1.0, 0.0, 0.0);
    let b = Vector3::new(0.0, -1.0, 0.0);
    let c = Vector3::new(0.0, 0.0, -1.0);
    assert!(!check_right_hand(&a, &b, &c));
}
