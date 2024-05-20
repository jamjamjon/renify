#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Task {
    Rename,
    Undo,
}

impl From<&str> for Task {
    fn from(s: &str) -> Self {
        match s {
            "Rename" => Self::Rename,
            "Undo with history" => Self::Undo,
            _ => todo!(),
        }
    }
}
