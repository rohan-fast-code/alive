// Alive desktop shell: main entry point. All real logic lives in the
// library file so the commands can be read and tested in one place.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    alive_app_lib::run();
}
