use super::*;
use image::Rgb32FImage;

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
