use std::process::ExitCode;

fn main() -> ExitCode {
    match ultraudit::run_from_env() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            anstream::eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
