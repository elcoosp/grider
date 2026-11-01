//! Command-line entry point of `grider`.
//!
//! All logic lives in [`grider::cli`] so it can be tested; this binary is a
//! thin wrapper that maps the result onto the process exit code.

fn main() -> std::process::ExitCode {
    grider::cli::main()
}
