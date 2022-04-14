use dyd::app::{App, AppResult};
use dyd::cli::CLI;
use dyd::event::{Event, EventHandler};
use dyd::handler::handle_key_events;
use dyd::manifest::Manifest;
use dyd::tui::Tui;

use tui::backend::CrosstermBackend;
use tui::Terminal;

fn main() -> AppResult<()> {
    let args = CLI::new();
    let manifest = Manifest::new(args)?;
    let mut app: App = manifest.into();

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;

        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}
