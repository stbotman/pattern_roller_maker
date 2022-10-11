use anyhow::{Context, Result};
use core::cmp::{max, min};
use image::imageops::FilterType;
use image::io::Reader;
use image::DynamicImage;

pub fn get_image_from_file(filename: &str) -> Result<DynamicImage> {
    let image_reader =
        Reader::open(&filename).with_context(|| format!("Failed to read file '{}'", filename))?;
    let image_reader_with_format = image_reader
        .with_guessed_format()
        .with_context(|| format!("Failed to read file '{}'", filename))?;
    let image = image_reader_with_format
        .decode()
        .with_context(|| format!("Failed to decode file '{}'", filename))?;
    Ok(image)
}

pub fn resize_image(
    image: DynamicImage,
    target_width: u32,
    target_height: u32,
    pixelated: bool,
) -> DynamicImage {
    let filter_type = if pixelated {
        FilterType::Nearest
    } else {
        if image.width() > target_width && image.height() > target_height {
            FilterType::Lanczos3
        } else {
            FilterType::CatmullRom
        }
    };
    image.resize_exact(target_width, target_height, filter_type)
}

pub fn image_to_vector(
    image: DynamicImage,
    inverted: bool,
    new_min: f64,
    new_max: f64,
) -> Vec<f64> {
    let gray_image = image.into_luma16();
    let image_vector = rescale_min_max(gray_image.into_vec(), inverted, new_min, new_max);
    image_vector
}

pub fn rescale_min_max(
    input_vector: Vec<u16>,
    inverted: bool,
    new_min: f64,
    new_max: f64,
) -> Vec<f64> {
    let mut global_max: u16 = std::u16::MIN;
    let mut gloabl_min: u16 = std::u16::MAX;
    for point in input_vector.iter() {
        global_max = max(*point, global_max);
        gloabl_min = min(*point, gloabl_min);
    }
    if gloabl_min != global_max {
        if inverted {
            (gloabl_min, global_max) = (global_max, gloabl_min);
        }
        let scale: f64 = (new_max - new_min) / (global_max as f64 - gloabl_min as f64);
        let output_vector: Vec<f64> = input_vector
            .iter()
            .map(|x| new_min + ({ *x as f64 } - gloabl_min as f64) * scale)
            .collect();
        output_vector
    } else {
        eprintln!("warning: Image is solid color");
        vec![0.5f64; input_vector.len()]
    }
}
