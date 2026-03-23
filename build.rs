const COMMANDS: &[&str] = &["pick_and_bookmark", "read_by_bookmark", "forget_bookmark"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .ios_path("ios")
        .build();
}
