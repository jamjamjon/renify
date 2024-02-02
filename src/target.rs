#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Target {
    Dir,
    File,
    /// TODO
    Symlink,
}

impl From<&str> for Target {
    fn from(s: &str) -> Self {
        match s {
            "File" | "file" => Self::File,
            "Directory" | "dir" | "Dir" | "Folder" => Self::Dir,
            "Symlink" | "link" => Self::Symlink,
            _ => todo!(),
        }
    }
}
