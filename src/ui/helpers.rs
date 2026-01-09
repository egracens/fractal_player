pub fn pick_file_dialog() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Audio", &["mp3", "flac"])
        .pick_file()
        .map(|path| path.display().to_string())
}
