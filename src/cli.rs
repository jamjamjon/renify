use crate::{Method, Target, Task};

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Source: File & directory
    pub input: String,

    /// Target: File & directory
    #[arg(short, long, value_enum, value_name("Target"))]
    pub target: Option<Target>,

    /// Tasks: rename & undo with history
    #[arg(short, long, value_enum)]
    pub task: Option<Task>,

    /// Methods for renaming
    #[arg(short, long, value_enum)]
    pub method: Option<Method>,

    /// Doing recursively or not
    #[arg(short, long)]
    pub recursive: Option<bool>,

    /// Depth when doing recursively
    #[arg(short, long)]
    pub depth: Option<usize>,

    /// The number of bit
    #[arg(short, long)]
    pub nbits: Option<usize>,

    /// Initial number
    #[arg(short, long)]
    pub start: Option<usize>,

    /// Text string for `Method::Prefix` & `Method::Append`
    #[arg(long)]
    pub with: Option<String>,

    /// Delimiter
    #[arg(long)]
    pub delimiter: Option<String>,

    /// Not preserving consistent file stems
    /// e.g. Files with the same filestem in the same folder should remain consistent after renaming
    #[arg(long)]
    pub indiscriminate: bool,

    // #[arg(short, long)]
    // pub hidden_included: bool,
    /// Execute without asking
    #[arg(short, long)]
    pub yes: bool,
}
