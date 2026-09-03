#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    qcyx_i18n::localize();
    qcyx_gui::run().unwrap();
}
