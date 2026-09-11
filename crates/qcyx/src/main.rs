#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    qcyx_i18n::localize();

    // `qcyx_gui::run` logs the error. An `unwrap` here aborted silently in
    // release builds (GUI subsystem, `panic = "abort"`).
    match qcyx_gui::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
