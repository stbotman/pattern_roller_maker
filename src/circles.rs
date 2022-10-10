use crate::vectors::Vector3;
use std::f64::consts::TAU;

pub struct CircleConverter {
    sin_cos_table: Vec<(f64, f64)>,
    pub axis_shift: f64,
    pub n_points: usize,
}

impl CircleConverter {
    pub fn new(n_points: usize, axis_shift: f64) -> CircleConverter {
        let phi_step = TAU / { n_points as f64 };
        let sin_cos_table = (0..n_points)
            .chain(Some(0))
            .map(|n| (n as f64 * phi_step).sin_cos())
            .collect::<Vec<_>>();
        CircleConverter {
            sin_cos_table: sin_cos_table,
            axis_shift: axis_shift,
            n_points: n_points,
        }
    }

    pub fn get_xy(&self, n: usize, rho: f64) -> (f64, f64) {
        let (sin_phi, cos_phi) = self.sin_cos_table[n];
        let x: f64 = rho * cos_phi + self.axis_shift;
        let y: f64 = rho * sin_phi + self.axis_shift;
        (x, y)
    }

    pub fn get_vector3(&self, n: usize, rho: f64, z: f64) -> Vector3 {
        let (x, y) = self.get_xy(n, rho);
        Vector3::new(x, y, z)
    }
}
