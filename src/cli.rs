use clap::builder::NonEmptyStringValueParser;
use clap::ArgAction::SetTrue;
use clap::{value_parser, Arg, ArgGroup, Command};

pub fn cli_command() -> Command<'static> {
    Command::new("Pattern Roller Maker")
        .author("Stepan Botman (github.com/stbotman)")
        .version(env!("CARGO_PKG_VERSION"))
        .about(concat!(
            "Simple tool to generate binary STL file for cylindrical pattern roller using input image, ",
            "so that image is etched onto its surface. ",
            "Either length ot diameter of roller should be specified, ",
            "remaining dimensions are calculated using image aspect ratio and stacking parameters. ",
            "Additionally, flat ends of roller can be specified to feature either pair of pins or through hole.",
        ))
        .arg(
            Arg::new("filename")
                .help("Filename of input image to be used as pattern")
                .required(true)
                .value_name("IMGFILE")
                .value_parser(NonEmptyStringValueParser::new())
                .index(1),
        )
        .arg(
            Arg::new("roller_diameter")
                .long("diameter")
                .short('d')
                .value_name("DIAM")
                .help("Roller body external diameter (length is auto calculated)")
                .takes_value(true)
                .value_parser(value_parser!(f64))
                .display_order(1),
        )
        .arg(
            Arg::new("roller_length")
                .long("length")
                .short('l')
                .value_name("LEN")
                .help("Roller body length (diameter is auto calculated)")
                .takes_value(true)
                .value_parser(value_parser!(f64))
                .display_order(2),
        )
        .arg(
            Arg::new("grid_step")
                .long("grid-step")
                .short('g')
                .value_name("STEP")
                .help("Distance between vertices on roller surface (input image is resized accordingly)")
                .takes_value(true)
                .value_parser(value_parser!(f64))
                .display_order(3),
        )
        .arg(
            Arg::new("relief_depth")
                .long("embossment-depth")
                .short('e')
                .value_name("DEPTH")
                .help("Maximum depth of surface pattern")
                .takes_value(true)
                .value_parser(value_parser!(f64))
                .display_order(4),
        )
        .arg(
            Arg::new("pin_diameter")
                .long("pin-diameter")
                .visible_alias("pd")
                .value_name("PDIAM")
                .help("Pin dimaeter (pins at both ends)")
                .takes_value(true)
                .value_parser(value_parser!(f64))
                .display_order(21),
        )
        .arg(
            Arg::new("pin_length")
                .long("pin-length")
                .visible_alias("pl")
                .value_name("PLEN")
                .help("Pin length (pins at both ends)")
                .takes_value(true)
                .value_parser(value_parser!(f64))
                .display_order(21),
        )
        .arg(
            Arg::new("channel_diameter")
                .long("channel-diameter")
                .visible_alias("cd")
                .value_name("CDIAM")
                .help("Channel diameter (coaxial cylindrical hole)")
                .takes_value(true)
                .value_parser(value_parser!(f64))
                .conflicts_with("pin_dimensions")
                .display_order(31),
        )
        .arg(
            Arg::new("output_filename")
                .long("output")
                .short('o')
                .value_name("STLFILE")
                .help("Output STL filename")
                .takes_value(true)
                .value_parser(NonEmptyStringValueParser::new())
                .display_order(41),
        )
        .arg(
            Arg::new("stack_vertical")
                .long("stack-vertical")
                .visible_alias("sv")
                .value_name("SVTIMES")
                .help("Stack copies of image vertically")
                .takes_value(true)
                .value_parser(value_parser!(u32).range(1..=1000))
                .display_order(51),
        )
        .arg(
            Arg::new("stack_horizontal")
                .long("stack-horizontal")
                .visible_alias("sh")
                .value_name("SHTIMES")
                .help("Stack copies of image horizontally")
                .takes_value(true)
                .value_parser(value_parser!(u32).range(1..=1000))
                .display_order(52),
        )
        .arg(
            Arg::new("pixelated")
                .long("pixelated")
                .short('p')
                .action(SetTrue)
                .help("Nearest-neighbor interpolation for image resize (if used)")
                .takes_value(false)
                .requires("grid_step")
                .display_order(100),
        )
        .arg(
            Arg::new("inverted")
                .long("inverted")
                .short('i')
                .action(SetTrue)
                .help("Invert image colors")
                .takes_value(false)
                .display_order(101),
        )
        .group(
            ArgGroup::new("roller_dimensions")
                .args(&["roller_diameter", "roller_length"])
                .required(true)
                .multiple(false),
        )
        .group(
            ArgGroup::new("pin_dimensions")
                .args(&["pin_diameter", "pin_length"])
                .requires_all(&["pin_diameter", "pin_length"])
                .required(false)
                .multiple(true)
                .conflicts_with("channel_diameter"),
        )
}
