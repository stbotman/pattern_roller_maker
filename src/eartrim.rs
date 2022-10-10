use crate::stl::STLFileWriter;
use crate::vectors::xy_scalar_product;
use crate::vectors::Vector3;
use anyhow::Result;

pub fn fill_polygon_by_ear_trimming(
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
    xy_scalar_product(&base_normal, &point_tip) > 0.0
}
