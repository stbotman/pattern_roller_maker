mod lib_arguments;
mod lib_geometry;
use lib_arguments::Parameters;
use lib_geometry::{make_pattern_roller, STLFileWriter};
use std::process::ExitCode;

fn actual_work() -> Result<(), anyhow::Error> {
    let parameters = Parameters::parse_arguments_and_file()?;
    parameters.print_summary()?;
    let stl_writer = STLFileWriter::new(&parameters)?;
    make_pattern_roller(&parameters, stl_writer)
}

fn main() -> ExitCode {
    match actual_work() {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {:#}", error);
            return ExitCode::FAILURE;
        }
    }
}
