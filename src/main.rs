mod app;
mod core;

use std::{
    env,
    io::{self, BufRead, BufReader, stdout},
    path::PathBuf,
};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::app::App;

fn main() -> anyhow::Result<()> {
    let path = resolve_input_file();
    let file = std::fs::File::open(&path)?;
    let reader = BufReader::new(file);
    let mut parser = core::parser::Parser::new();
    for line in reader.lines() {
        let line = line?;
        parser = parser.consume(line)?;
    }
    let merge_file = parser.into_merge_file()?;

    let mut app = App::new(merge_file, path.clone())?;

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn resolve_input_file() -> PathBuf {
    if let Ok(merged) = std::env::var("MERGED") {
        return PathBuf::from(merged);
    }

    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => PathBuf::from(&args[1]),
        5 => PathBuf::from(&args[4]),
        _ => {
            eprintln!("Usage: tmerge <file>");
            eprintln!("       tmerge BASE LOCAL REMOTE MERGED");
            std::process::exit(1);
        }
    }
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| app::ui::render(app, frame))?;
        app::event::handle_events(app)?;

        if app.force_redraw {
            app.force_redraw = false;
            terminal.clear()?;
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
