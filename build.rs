const COMMANDS: &[&str] = &[
    "pick_and_bookmark",
    "pick_folder_and_bookmark",
    "list_by_folder_bookmark",
    "create_folder_by_folder_bookmark",
    "create_markdown_file_by_folder_bookmark",
    "rename_by_folder_bookmark",
    "move_by_folder_bookmark",
    "delete_by_folder_bookmark",
    "read_by_bookmark",
    "write_by_bookmark",
    "read_by_folder_bookmark",
    "read_binary_by_folder_bookmark",
    "write_by_folder_bookmark",
    "export_file",
    "export_pdf",
    "forget_bookmark",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).ios_path("ios").build();
}
