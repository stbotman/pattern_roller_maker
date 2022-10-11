use crate::circles::CircleConverter;
use crate::eartrim::fill_polygon_by_ear_trimming;
use crate::parameters::{Parameters, RollerEnd};
use crate::split::split_quad_optimal;
use crate::stl::STLFileWriter;
use crate::vectors::Vector3;
use anyhow::Result;

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
