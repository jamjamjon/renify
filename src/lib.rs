use indicatif::{ProgressBar, ProgressState, ProgressStyle};

mod cli;
mod impl_;
mod method;
mod target;
mod task;

pub use cli::Cli;
pub use method::Method;
pub use target::Target;
pub use task::Task;

/// illegal characters
const INVALID_CHARS: &str = "<>:/\"|?*'`";
const BIT_MAX: usize = 20;
const CROSS_MARK: &str = "❌";
const CHECK_MARK: &str = "✅";

fn build_progressbar(size: u64, prefix: &str) -> ProgressBar {
    let pb = ProgressBar::new(size);
    pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green.bold} {prefix:.bold} [{bar:.blue.bright.bold/white.dim}] {human_pos}/{human_len} ({percent}% | {eta} | {elapsed_precise})"
            )
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write | write!(w, "{:.2}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));
    pb.set_prefix(prefix.to_string());
    pb
}
