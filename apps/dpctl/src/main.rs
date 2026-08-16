use std::process::ExitCode;

fn main() -> ExitCode {
    let output = dpctl::run(std::env::args().skip(1));
    print!("{}", output.stdout);
    eprint!("{}", output.stderr);
    ExitCode::from(output.exit_code)
}
