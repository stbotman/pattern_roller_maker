use crate::cli::cli_command;
use crate::image::{get_image_from_file, image_to_vector, resize_image};
use anyhow::{ensure, Context, Result};
use clap::ArgMatches;
use image::DynamicImage;
#[cfg(test)]
use image::Rgb32FImage;
use std::f64::consts::{PI, TAU};

pub struct Parameters {
    pub output_filename: String,
    pub radii_vector: Vec<f64>,
    pub image_width: u32,
    pub image_height: u32,
    pub stack_horizontal: u32,
    pub stack_vertical: u32,
    pub roller_diameter: f64,
    pub roller_length: f64,
    pub relief_depth: f64,
    pub grid_step: f64,
    pub roller_end: RollerEnd,
}

pub enum RollerEnd {
    Flat,
    Pin {
        pin_diameter: f64,
        pin_length: f64,
        circle_points: u32,
    },
    Channel {
        channel_diameter: f64,
        circle_points: u32,
    },
}

impl Parameters {
    pub fn parse_arguments_and_file() -> Result<Parameters> {
        let matches = cli_command().get_matches();
        let input_filename = matches.get_one::<String>("filename").unwrap();
        let raw_image = get_image_from_file(input_filename)?;
        parse_macthes(matches, raw_image)
    }

    pub fn circle_points(&self) -> u32 {
        self.image_width * self.stack_horizontal
    }

    pub fn faces_count(&self) -> Result<u32> {
        const OVERFLOW_ERROR_TEXT: &str =
            "Overflow in STL face counter: resulting model is too big";
        let full_body_width_points = self.image_width * self.stack_horizontal;
        let full_body_height_points = self.image_height * self.stack_vertical - 1;
        let full_body_points = full_body_width_points
            .checked_mul(full_body_height_points)
            .with_context(|| OVERFLOW_ERROR_TEXT)?;
        let full_body_faces = 2u32
            .checked_mul(full_body_points)
            .with_context(|| OVERFLOW_ERROR_TEXT)?;
        let ends_faces_count = match self.roller_end {
            RollerEnd::Flat => 2 * full_body_width_points,
            RollerEnd::Pin { circle_points, .. } => 2 * full_body_width_points + 8 * circle_points,
            RollerEnd::Channel { circle_points, .. } => {
                2 * full_body_width_points + 4 * circle_points
            }
        };
        let n_faces = full_body_faces
            .checked_add(ends_faces_count)
            .with_context(|| OVERFLOW_ERROR_TEXT)?;
        Ok(n_faces)
    }

    pub fn get_rho(&self, i: usize, j: usize) -> f64 {
        let index = j * { self.image_width as usize } + i;
        self.radii_vector[index]
    }

    pub fn get_rho_looped(&self, i_raw: i32, j_raw: i32) -> f64 {
        let i = i_raw.rem_euclid(self.image_width as i32) as usize;
        let j = j_raw.rem_euclid(self.image_height as i32) as usize;
        self.get_rho(i, j)
    }

    pub fn get_image_topline(&self) -> &[f64] {
        &self.radii_vector[..{ self.image_width as usize }]
    }

    pub fn get_image_botline(&self) -> &[f64] {
        &self.radii_vector[self.radii_vector.len() - { self.image_width as usize }..]
    }
}

impl Parameters {
    fn bytes_estimate(&self) -> Result<u64> {
        let n_faces = self.faces_count()? as u64;
        Ok(50 * n_faces + 84)
    }

    fn format_bytes_size(bytes_count: u64) -> String {
        let magnitude = { bytes_count as f64 }.log2() as u32 / 10;
        let (unit, base) = match magnitude {
            0 => ("B", 1),
            1 => ("KiB", u32::pow(2, 10)),
            2 => ("MiB", u32::pow(2, 20)),
            3.. => ("GiB", u32::pow(2, 30)),
        };
        let size_string = match magnitude {
            0 => format!("{bytes_count} {unit}"),
            1.. => {
                let value = { bytes_count as f64 } / { base as f64 };
                format!("{value:.2} {unit}")
            }
        };
        size_string
    }

    pub fn print_summary(&self) -> Result<()> {
        let size_string = Parameters::format_bytes_size(self.bytes_estimate()?);
        println!(
            "length: {:.2} diameter: {:.2} filesize: {}",
            self.roller_length, self.roller_diameter, size_string
        );
        Ok(())
    }
}

fn parse_macthes(matches: ArgMatches, raw_image: DynamicImage) -> Result<Parameters> {
    let stack_horizontal: u32 = *matches.get_one::<u32>("stack_horizontal").unwrap_or(&1u32);
    let stack_vertical: u32 = *matches.get_one::<u32>("stack_vertical").unwrap_or(&1u32);
    let surface_width_px: u32 = raw_image.width() * stack_horizontal;
    let surface_height_px: u32 = raw_image.height() * stack_vertical;
    let surface_aspect_ratio: f64 = { surface_width_px as f64 } / { surface_height_px as f64 };
    let (diameter, length, pixel_size) = if matches.contains_id("roller_diameter") {
        let diameter: f64 = *matches.get_one::<f64>("roller_diameter").unwrap();
        let pixel_size: f64 = PI * diameter / { surface_width_px as f64 };
        let length: f64 = PI * diameter / surface_aspect_ratio;
        (diameter, length, pixel_size)
    } else {
        let length: f64 = *matches.get_one::<f64>("roller_length").unwrap();
        let pixel_size: f64 = length / { surface_height_px as f64 };
        let diameter: f64 = length * surface_aspect_ratio / PI;
        (diameter, length, pixel_size)
    };
    ensure!(
        diameter > 0.0 && length > 0.0,
        "All roller dimensions should be greater than zero"
    );
    let relief_depth: f64 = *matches
        .get_one::<f64>("relief_depth")
        .unwrap_or(&(0.02 * &diameter));
    ensure!(
        relief_depth > 0.0,
        "Relief depth should be greater than zero"
    );
    ensure!(
        diameter > 2.0 * relief_depth,
        "Relief depth ({}) should be less than radius ({})",
        relief_depth,
        diameter * 0.5
    );
    let (image, image_width, image_height, grid_step) = if matches.contains_id("grid_step") {
        let grid_step: f64 = *matches.get_one::<f64>("grid_step").unwrap();
        let scale = pixel_size / grid_step;
        let target_width = (scale * { raw_image.width() as f64 }).round() as u32;
        let target_height = (scale * { raw_image.height() as f64 }).round() as u32;
        let pixelated = matches.get_flag("pixelated");
        let resized_image = resize_image(raw_image, target_width, target_height, pixelated);
        (resized_image, target_width, target_height, grid_step)
    } else {
        let width = raw_image.width();
        let height = raw_image.height();
        let grid_step = length / { surface_height_px as f64 };
        (raw_image, width, height, grid_step)
    };
    let inverted = matches.get_flag("inverted");
    let radii_vector = image_to_vector(
        image,
        inverted,
        diameter * 0.5 - relief_depth,
        diameter * 0.5,
    );
    let roller_end = if matches.contains_id("pin_diameter") {
        let pin_diameter = *matches.get_one::<f64>("pin_diameter").unwrap();
        let pin_length = *matches.get_one::<f64>("pin_length").unwrap();
        ensure!(pin_length > 0.0, "Pin length should be greater than zero");
        ensure!(
            diameter - 2.0 * relief_depth > pin_diameter,
            "Pin diameter ({}) is too big (should be < {})",
            pin_diameter,
            diameter - 2.0 * relief_depth
        );
        RollerEnd::Pin {
            pin_diameter: pin_diameter,
            pin_length: pin_length,
            circle_points: (TAU * pin_diameter / grid_step).round() as u32,
        }
    } else if matches.contains_id("channel_diameter") {
        let channel_diameter = *matches.get_one::<f64>("channel_diameter").unwrap();
        ensure!(
            diameter - 2.0 * relief_depth > channel_diameter,
            "Channel diameter ({}) is too big (should be < {})",
            diameter,
            channel_diameter - 2.0 * relief_depth
        );
        RollerEnd::Channel {
            channel_diameter: channel_diameter,
            circle_points: (TAU * channel_diameter / grid_step).round() as u32,
        }
    } else {
        RollerEnd::Flat
    };
    let output_filename: String = if matches.contains_id("output_filename") {
        matches
            .get_one::<String>("output_filename")
            .unwrap()
            .clone()
    } else {
        let mut default_filename = matches.get_one::<String>("filename").unwrap().clone();
        default_filename.push_str(".stl");
        default_filename
    };
    Ok(Parameters {
        output_filename: output_filename,
        radii_vector: radii_vector,
        image_width: image_width,
        image_height: image_height,
        stack_horizontal: stack_horizontal,
        stack_vertical: stack_vertical,
        roller_diameter: diameter,
        roller_length: length,
        relief_depth: relief_depth,
        grid_step: grid_step,
        roller_end: roller_end,
    })
}

#[cfg(test)]
fn test_cli_arguments(command_string: &str) -> Result<Parameters, anyhow::Error> {
    let image = DynamicImage::ImageRgb32F(Rgb32FImage::new(10, 10));
    let arguments: Vec<&str> = command_string.split_whitespace().collect();
    let matches = cli_command().try_get_matches_from(arguments)?;
    let parameters = parse_macthes(matches, image)?;
    Ok(parameters)
}

#[test]
fn test_conflicting_arguments() {
    let parameters = test_cli_arguments("img2roller -l 1 -d 1 test.png");
    assert!(parameters.is_err());
    let parameters = test_cli_arguments("img2roller -l 1 --pd 1 --pl 1 --cd 1 test.png");
    assert!(parameters.is_err());
}

#[test]
fn test_missing_required_arguments() {
    let parameters = test_cli_arguments("img2roller test.png");
    assert!(parameters.is_err());
    let parameters = test_cli_arguments("img2roller -l 1 --pd 1 test.png");
    assert!(parameters.is_err());
    let parameters = test_cli_arguments("img2roller -l 1 --pl 1 test.png");
    assert!(parameters.is_err());
    let parameters = test_cli_arguments("img2roller -l 1 -p test.png");
    assert!(parameters.is_err());
}

#[test]
fn test_invalid_arguments() {
    let parameters = test_cli_arguments("img2roller -d 2 -e 1 test.png");
    assert!(parameters.is_err());
    let parameters = test_cli_arguments("img2roller -d 2 -e 0 test.png");
    assert!(parameters.is_err());
    let parameters = test_cli_arguments("img2roller -d 0.0 test.png");
    assert!(parameters.is_err());
    let parameters = test_cli_arguments("img2roller -l 0.0 test.png");
    assert!(parameters.is_err());
    let parameters = test_cli_arguments("img2roller -d 1 --pd 1 --pl 1 test.png");
    assert!(parameters.is_err());
    let parameters = test_cli_arguments("img2roller -d 1 --cd 1 test.png");
    assert!(parameters.is_err());
}

#[test]
fn test_dimensions_arguments() {
    let parameters = test_cli_arguments("img2roller -d 1 test.png").unwrap();
    assert_eq!(parameters.roller_length, PI);
    let parameters = test_cli_arguments("img2roller -l 1 test.png").unwrap();
    assert_eq!(parameters.roller_diameter, 1.0 / PI);
    let parameters = test_cli_arguments("img2roller -d 1 --sv 10 test.png").unwrap();
    assert_eq!(parameters.roller_length, 10.0 * PI);
    let parameters = test_cli_arguments("img2roller -d 1 --sh 10 test.png").unwrap();
    assert_eq!(parameters.roller_length, PI * 0.1);
}
