// Prevent console on Windows release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    vibestation_app_lib::run();
}
