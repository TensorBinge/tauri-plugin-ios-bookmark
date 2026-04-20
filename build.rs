const COMMANDS: &[&str] = &[
    "pick_and_bookmark",
    "pick_folder_and_bookmark",
    "read_by_bookmark",
    "write_by_bookmark",
    "read_by_folder_bookmark",
    "write_by_folder_bookmark",
    "export_file",
    "export_pdf",
    "forget_bookmark",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).ios_path("ios").build();
}
