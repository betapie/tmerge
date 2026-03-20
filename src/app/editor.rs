use std::io::{BufWriter, Write};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tempfile::NamedTempFile;

#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Extenal editor failure: {0}")]
    ExternalEditorError(String),
}

pub fn edit(lines: &[String]) -> Result<Vec<String>, EditorError> {
    let tempfile = NamedTempFile::new()?;
    let mut writer = BufWriter::new(tempfile.as_file());
    for line in lines {
        writeln!(writer, "{}", line)?;
    }
    writer.flush()?;

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;

    let editor = std::env::var("EDITOR").unwrap_or("vi".to_string());
    let status = std::process::Command::new(&editor)
        .arg(tempfile.path())
        .status()?;

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;

    if !status.success() {
        return Err(EditorError::ExternalEditorError(format!(
            "{} exited with non-zero status",
            editor
        )));
    }

    let read_lines = std::fs::read_to_string(tempfile.path())?
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    Ok(read_lines)
}
