mod circles;
mod cli;
mod construct;
mod eartrim;
mod image;
mod parameters;
mod split;
mod stl;
mod vectors;
use crate::stl::STLFileWriter;
use construct::make_pattern_roller;
use parameters::Parameters;
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
