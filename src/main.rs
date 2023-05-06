use std::process::ExitCode;
fn main() -> ExitCode {
    if let Err(e) = tuicom::run_app() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
