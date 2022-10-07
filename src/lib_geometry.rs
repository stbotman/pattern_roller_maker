use crate::lib_arguments::{Parameters, RollerEnd};
use anyhow::{Context, Error, Result};
use std::f64::consts::TAU;
use std::f64::EPSILON;
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};

pub struct STLFileWriter {
    buffered_file: BufWriter<File>,
    faces_count: u32,
}

impl STLFileWriter {
    const HEADER_SIZE: usize = 80;
    const HEADER_TEXT: [u8; 14] = *b"pattern roller";
    const SPACER: [u8; 2] = [0u8; 2];

    fn write_data(&mut self, data: &[u8]) -> Result<()> {
        self.buffered_file.write_all(data).map_err(Error::from)
    }

    fn write_header(&mut self) -> Result<()> {
        let mut header: Vec<u8> = Vec::with_capacity(STLFileWriter::HEADER_SIZE);
        header.extend_from_slice(&STLFileWriter::HEADER_TEXT);
        header.resize(STLFileWriter::HEADER_SIZE, 0u8);
        self.write_data(&header)
    }

    fn write_n_faces(&mut self) -> Result<()> {
        self.write_data(&self.faces_count.to_le_bytes())
    }

    fn write_face(
        &mut self,
        vec_n: &Vector3,
        vec_a: &Vector3,
        vec_b: &Vector3,
        vec_c: &Vector3,
    ) -> Result<()> {
        if cfg!(debug_assertions) {
            debug_face_data(vec_n, vec_a, vec_b, vec_c);
            self.faces_count -= 1;
        }
        self.write_data(&vec_n.to_binary())?;
        self.write_data(&vec_a.to_binary())?;
        self.write_data(&vec_b.to_binary())?;
        self.write_data(&vec_c.to_binary())?;
        self.write_data(&STLFileWriter::SPACER)
    }

    fn write_face_auto_normal(
        &mut self,
        vec_a: &Vector3,
        vec_b: &Vector3,
        vec_c: &Vector3,
    ) -> Result<()> {
        let vec_n = face_normal(vec_a, vec_b, vec_c);
        self.write_face(&vec_n, vec_a, vec_b, vec_c)
    }

    pub fn new(params: &Parameters) -> Result<STLFileWriter> {
        let filename = params.output_filename.as_str();
        let file = File::create(filename)
            .with_context(|| format!("Failed to open file '{}' for writing", filename))?;
        let buffered_file = BufWriter::new(file);
        let n_faces = params.faces_count()?;
        let mut stl_writer = STLFileWriter {
            buffered_file: buffered_file,
            faces_count: n_faces,
        };
        stl_writer.write_header()?;
        stl_writer.write_n_faces()?;
        Ok(stl_writer)
    }
}

#[cfg(debug_assertions)]
impl Drop for STLFileWriter {
    fn drop(&mut self) {
        assert!(
            self.faces_count == 0,
            "Faces count mismatch: stl file was not fully written"
        );
    }
}

pub fn make_pattern_roller(params: &Parameters, mut stl_writer: STLFileWriter) -> Result<()> {
    let big_circle = CircleConverter::new(
        params.circle_points() as usize,
        params.roller_diameter * 0.5,
    );
    make_cylinder_patterned(&mut stl_writer, &params, &big_circle)?;
    match params.roller_end {
        RollerEnd::Flat => make_lids_solid(&mut stl_writer, &params, big_circle),
        RollerEnd::Pin {
            circle_points,
            pin_diameter,
            pin_length,
        } => {
            let small_circle =
                CircleConverter::new(circle_points as usize, params.roller_diameter * 0.5);
            make_pins(
                &mut stl_writer,
                &params,
                &small_circle,
                pin_diameter,
                pin_length,
            )?;
            make_lids_holed(
                &mut stl_writer,
                &params,
                &big_circle,
                &small_circle,
                pin_diameter,
                pin_length,
            )
        }
        RollerEnd::Channel {
            circle_points,
            channel_diameter,
        } => {
            let small_circle =
                CircleConverter::new(circle_points as usize, params.roller_diameter * 0.5);
            make_channel(&mut stl_writer, &params, &small_circle, channel_diameter)?;
            make_lids_holed(
                &mut stl_writer,
                &params,
                &big_circle,
                &small_circle,
                channel_diameter,
                0.0,
            )
        }
    }
}

fn make_cylinder_patterned(
    stl_writer: &mut STLFileWriter,
    params: &Parameters,
    circle: &CircleConverter,
) -> Result<()> {
    let width = params.image_width as usize;
    let height = params.image_height as usize;
    let hstack = params.stack_horizontal as usize;
    let vstack = params.stack_vertical as usize;
    let z_max = match params.roller_end {
        RollerEnd::Flat => params.roller_length,
        RollerEnd::Channel { .. } => params.roller_length,
        RollerEnd::Pin { pin_length, .. } => params.roller_length + pin_length,
    };
    let z_step =
        params.roller_length / { (params.image_height * params.stack_vertical - 1) as f64 };
    for i in 0..width {
        for j in 0..height {
            let (tlbr_split, rho_tl, rho_tr, rho_bl, rho_br) = split_quad_optimal(params, i, j);
            for p in 0..hstack {
                let (x_tl, y_tl) = circle.get_xy(i + p * width, rho_tl);
                let (x_tr, y_tr) = circle.get_xy(i + p * width + 1, rho_tr);
                let (x_bl, y_bl) = circle.get_xy(i + p * width, rho_bl);
                let (x_br, y_br) = circle.get_xy(i + p * width + 1, rho_br);
                for q in 0..vstack {
                    if j == height - 1 && q == vstack - 1 {
                        continue;
                    };
                    let z_t = z_max - { (j + height * q) as f64 } * z_step;
                    let z_b = z_t - z_step;
                    let point_tl = Vector3::new(x_tl, y_tl, z_t);
                    let point_bl = Vector3::new(x_bl, y_bl, z_b);
                    let point_tr = Vector3::new(x_tr, y_tr, z_t);
                    let point_br = Vector3::new(x_br, y_br, z_b);
                    if tlbr_split {
                        stl_writer.write_face_auto_normal(&point_tl, &point_br, &point_tr)?;
                        stl_writer.write_face_auto_normal(&point_bl, &point_br, &point_tl)?;
                    } else {
                        stl_writer.write_face_auto_normal(&point_bl, &point_tr, &point_tl)?;
                        stl_writer.write_face_auto_normal(&point_bl, &point_br, &point_tr)?;
                    };
                }
            }
        }
    }
    Ok(())
}

fn split_quad_optimal(params: &Parameters, i: usize, j: usize) -> (bool, f64, f64, f64, f64) {
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

fn make_lids_solid(
    stl_writer: &mut STLFileWriter,
    params: &Parameters,
    circle: CircleConverter,
) -> Result<()> {
    let z_max = params.roller_length;
    let top_radii = params.get_image_topline();
    let bot_radii = params.get_image_botline();
    let mut top_point_old: Vector3;
    let mut bot_point_old: Vector3;
    let top_center = Vector3::new(circle.axis_shift, circle.axis_shift, z_max);
    let bot_center = Vector3::new(circle.axis_shift, circle.axis_shift, 0.0);
    let mut top_point_new = circle.get_vector3(0, top_radii[0], z_max);
    let mut bot_point_new = circle.get_vector3(0, bot_radii[0], 0.0);
    for i in 1..=circle.n_points {
        top_point_old = top_point_new;
        bot_point_old = bot_point_new;
        top_point_new = circle.get_vector3(i, top_radii[i % top_radii.len()], z_max);
        bot_point_new = circle.get_vector3(i, bot_radii[i % bot_radii.len()], 0.0);
        stl_writer.write_face(&Vector3::UP, &top_center, &top_point_old, &top_point_new)?;
        stl_writer.write_face(&Vector3::DOWN, &bot_center, &bot_point_new, &bot_point_old)?;
    }
    Ok(())
}

fn make_channel(
    stl_writer: &mut STLFileWriter,
    params: &Parameters,
    circle: &CircleConverter,
    channel_diameter: f64,
) -> Result<()> {
    let z_max = params.roller_length;
    let channel_radius = channel_diameter * 0.5;
    let mut top_point_old: Vector3;
    let mut bot_point_old: Vector3;
    let mut top_point_new = circle.get_vector3(0, channel_radius, z_max);
    let mut bot_point_new = circle.get_vector3(0, channel_radius, 0.0);
    for i in 1..=circle.n_points {
        top_point_old = top_point_new;
        bot_point_old = bot_point_new;
        top_point_new = circle.get_vector3(i, channel_radius, z_max);
        bot_point_new = circle.get_vector3(i, channel_radius, 0.0);
        let normal = Vector3::from_points(&top_point_old, &top_point_new).xy_perp_clockwise();
        stl_writer.write_face(&normal, &top_point_old, &top_point_new, &bot_point_old)?;
        stl_writer.write_face(&normal, &bot_point_old, &top_point_new, &bot_point_new)?;
    }
    Ok(())
}

fn make_pins(
    stl_writer: &mut STLFileWriter,
    params: &Parameters,
    circle: &CircleConverter,
    pin_diameter: f64,
    pin_length: f64,
) -> Result<()> {
    let z_max = params.roller_length + 2.0 * pin_length;
    let top_center = Vector3::new(circle.axis_shift, circle.axis_shift, z_max);
    let bot_center = Vector3::new(circle.axis_shift, circle.axis_shift, 0.0);
    let pin_radius = pin_diameter * 0.5;
    let (mut x_old, mut y_old): (f64, f64);
    let (mut x_new, mut y_new) = circle.get_xy(0, pin_radius);
    for i in 1..=circle.n_points {
        (x_old, y_old) = (x_new, y_new);
        (x_new, y_new) = circle.get_xy(i, pin_radius);
        {
            let top_point_1 = Vector3::new(x_old, y_old, z_max);
            let top_point_2 = Vector3::new(x_new, y_new, z_max);
            stl_writer.write_face(&Vector3::UP, &top_point_1, &top_point_2, &top_center)?;
            let top_point_3 = Vector3::new(x_old, y_old, z_max - pin_length);
            let top_point_4 = Vector3::new(x_new, y_new, z_max - pin_length);
            let normal = Vector3::from_points(&top_point_2, &top_point_1).xy_perp_clockwise();
            stl_writer.write_face(&normal, &top_point_3, &top_point_2, &top_point_1)?;
            stl_writer.write_face(&normal, &top_point_2, &top_point_3, &top_point_4)?;
        }
        {
            let bot_point_1 = Vector3::new(x_old, y_old, 0.0);
            let bot_point_2 = Vector3::new(x_new, y_new, 0.0);
            stl_writer.write_face(&Vector3::DOWN, &bot_center, &bot_point_2, &bot_point_1)?;
            let bot_point_3 = Vector3::new(x_old, y_old, pin_length);
            let bot_point_4 = Vector3::new(x_new, y_new, pin_length);
            let normal = Vector3::from_points(&bot_point_2, &bot_point_1).xy_perp_clockwise();
            stl_writer.write_face(&normal, &bot_point_1, &bot_point_2, &bot_point_3)?;
            stl_writer.write_face(&normal, &bot_point_4, &bot_point_3, &bot_point_2)?;
        }
    }
    Ok(())
}

fn make_lids_holed(
    stl_writer: &mut STLFileWriter,
    params: &Parameters,
    big_circle: &CircleConverter,
    small_circle: &CircleConverter,
    inner_dimaeter: f64,
    z_shift: f64,
) -> Result<()> {
    let radii_top = params.get_image_topline();
    let radii_bot = params.get_image_botline();
    let z_top = z_shift + params.roller_length;
    let z_bot = z_shift;
    let inner_radius = inner_dimaeter * 0.5;
    let step_scale = { big_circle.n_points as f64 } / { small_circle.n_points as f64 };
    let polygon_capacity = { step_scale.ceil() as usize } + 3;
    let (mut x_old, mut y_old): (f64, f64);
    let (mut x_new, mut y_new) = small_circle.get_xy(0, inner_radius);
    let mut n_start: usize;
    let mut n_end: usize = 0;
    for i in 1..=small_circle.n_points {
        (x_old, y_old) = (x_new, y_new);
        (x_new, y_new) = small_circle.get_xy(i, inner_radius);
        n_start = n_end;
        if i != small_circle.n_points {
            n_end = ({ i as f64 } * step_scale).round() as usize;
        } else {
            n_end = big_circle.n_points;
        };
        {
            let mut top_polygon: Vec<Vector3> = Vec::with_capacity(polygon_capacity);
            top_polygon.push(Vector3::new(x_new, y_new, z_top));
            top_polygon.extend(
                (n_start..=n_end)
                    .rev()
                    .map(|n| big_circle.get_vector3(n, radii_top[n % radii_top.len()], z_top)),
            );
            top_polygon.push(Vector3::new(x_old, y_old, z_top));
            fill_polygon_by_ear_trimming(stl_writer, top_polygon, true)?;
        }
        {
            let mut bot_polygon: Vec<Vector3> = Vec::with_capacity(polygon_capacity);
            bot_polygon.push(Vector3::new(x_new, y_new, z_bot));
            bot_polygon.extend(
                (n_start..=n_end)
                    .rev()
                    .map(|n| big_circle.get_vector3(n, radii_bot[n % radii_bot.len()], z_bot)),
            );
            bot_polygon.push(Vector3::new(x_old, y_old, z_bot));
            fill_polygon_by_ear_trimming(stl_writer, bot_polygon, false)?;
        }
    }
    Ok(())
}

fn fill_polygon_by_ear_trimming(
    stl_writer: &mut STLFileWriter,
    mut polygon_points: Vec<Vector3>,
    normal_up: bool,
) -> Result<()> {
    let normal: Vector3 = if normal_up {
        Vector3::UP
    } else {
        Vector3::DOWN
    };
    'outer: while polygon_points.len() >= 3 {
        for i in (1..polygon_points.len() - 1).rev() {
            if trinagle_is_ear(
                &polygon_points[i - 1],
                &polygon_points[i],
                &polygon_points[i + 1],
            ) {
                if normal_up {
                    stl_writer.write_face(
                        &normal,
                        &polygon_points[i - 1],
                        &polygon_points[i + 1],
                        &polygon_points[i],
                    )?;
                } else {
                    stl_writer.write_face(
                        &normal,
                        &polygon_points[i],
                        &polygon_points[i + 1],
                        &polygon_points[i - 1],
                    )?;
                };
                polygon_points.remove(i);
                continue 'outer;
            }
        }
        panic!("Failed to triangulate polygon iteratively");
    }
    Ok(())
}

fn trinagle_is_ear(point_left: &Vector3, point_middle: &Vector3, point_right: &Vector3) -> bool {
    let base_normal = Vector3::from_points(point_left, point_right).xy_perp_clockwise();
    let point_tip = Vector3::from_points(point_left, point_middle);
    let xy_scalar_product = base_normal.x * point_tip.x + base_normal.y * point_tip.y;
    xy_scalar_product > 0.0
}

pub struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector3 {
    const UP: Vector3 = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    const DOWN: Vector3 = Vector3 {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };

    const ZERO: Vector3 = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 { x: x, y: y, z: z }
    }

    fn from_points(origin: &Vector3, end: &Vector3) -> Vector3 {
        Vector3 {
            x: end.x - origin.x,
            y: end.y - origin.y,
            z: end.z - origin.z,
        }
    }

    fn normalize(mut self) -> Self {
        let scale: f64 = (self.x.powi(2) + self.y.powi(2) + self.z.powi(2))
            .sqrt()
            .recip();
        self.x = self.x * scale;
        self.y = self.y * scale;
        self.z = self.z * scale;
        self
    }

    fn xy_perp_clockwise(self) -> Vector3 {
        Vector3 {
            x: -self.y,
            y: self.x,
            z: self.z,
        }
    }

    fn to_binary(&self) -> [u8; 3 * 4] {
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

struct CircleConverter {
    sin_cos_table: Vec<(f64, f64)>,
    axis_shift: f64,
    n_points: usize,
}

impl CircleConverter {
    fn new(n_points: usize, axis_shift: f64) -> CircleConverter {
        let mut sin_cos_table: Vec<(f64, f64)> = Vec::with_capacity(n_points + 1);
        let mut phi: f64;
        let phi_step = TAU / { n_points as f64 };
        for n in 0..n_points {
            phi = { n as f64 } * phi_step;
            sin_cos_table.push(phi.sin_cos());
        }
        sin_cos_table.push(sin_cos_table[0]);
        CircleConverter {
            sin_cos_table: sin_cos_table,
            axis_shift: axis_shift,
            n_points: n_points,
        }
    }

    fn get_xy(&self, n: usize, rho: f64) -> (f64, f64) {
        let (sin_phi, cos_phi) = self.sin_cos_table[n];
        let x: f64 = rho * cos_phi + self.axis_shift;
        let y: f64 = rho * sin_phi + self.axis_shift;
        (x, y)
    }

    fn get_vector3(&self, n: usize, rho: f64, z: f64) -> Vector3 {
        let (x, y) = self.get_xy(n, rho);
        Vector3 { x: x, y: y, z: z }
    }
}

fn cross_product(vec_a: Vector3, vec_b: Vector3) -> Vector3 {
    let x = vec_a.y * vec_b.z - vec_a.z * vec_b.y;
    let y = vec_a.z * vec_b.x - vec_a.x * vec_b.z;
    let z = vec_a.x * vec_b.y - vec_a.y * vec_b.x;
    Vector3::new(x, y, z)
}

fn face_normal(vec_a: &Vector3, vec_b: &Vector3, vec_c: &Vector3) -> Vector3 {
    let vec_ab = Vector3::from_points(vec_a, vec_b);
    let vec_ac = Vector3::from_points(vec_a, vec_c);
    let vec_normal = cross_product(vec_ab, vec_ac).normalize();
    vec_normal
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

fn debug_face_data(vec_n: &Vector3, vec_a: &Vector3, vec_b: &Vector3, vec_c: &Vector3) {
    assert!(
        vec_a != vec_b && vec_b != vec_c && vec_c != vec_a && vec_n != &Vector3::ZERO,
        "Encountered degenerate face: a:{:?} b:{:?} c:{:?} n:{:?}",
        &vec_a,
        &vec_b,
        &vec_c,
        &vec_n
    );
    assert!(
        check_right_hand(
            &Vector3::from_points(vec_b, vec_c),
            &Vector3::from_points(vec_b, vec_a),
            &vec_n
        ),
        "Encountered inverted normal: a:{:?} b:{:?} c:{:?} n:{:?}",
        &vec_a,
        &vec_b,
        &vec_c,
        &vec_n
    );
}

fn check_right_hand(vec_a: &Vector3, vec_b: &Vector3, vec_c: &Vector3) -> bool {
    let det: f64 =
        vec_a.x * vec_b.y * vec_c.z + vec_a.y * vec_b.z * vec_c.x + vec_a.z * vec_b.x * vec_c.y
            - vec_a.z * vec_b.y * vec_c.x
            - vec_a.y * vec_b.x * vec_c.z
            - vec_a.x * vec_b.z * vec_c.y;
    det > 0.0
}

#[cfg(test)]
#[path = "test_lib_geometry.rs"]
mod test_lib_geometry;
