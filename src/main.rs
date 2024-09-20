use std::{fs, io};

use backend::cliargs::{self, CLIArgs};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    frontend::app::{App, AppResult},
    frontend::event::{Event, EventHandler},
    frontend::handler::handle_key_events,
    frontend::tui::Tui,
};

use sarge::prelude::*;

pub mod backend;
pub mod frontend;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Parse CLI arguments
    let parsed_args = CLIArgs::parse_cli_args();

    // Print help if -h/--help flag is passed and exit
    if parsed_args.helparg {
        println!("{}", cliargs::help_func());
        std::process::exit(0);
    }

    if parsed_args.versionarg {
        // println!("Version Zero");
        println!("{}", cliargs::version_func());
        std::process::exit(0);
    }
    // TODO: Implement logic for CLI arguments/options which need to be handled
    // before the TUI is started

    // Create an application.
    let mut app = App::new();

    // TEST: Get Data from main bibliography
    // let bibfile = fs::read_to_string("test.bib").unwrap();
    // let biblio = Bibliography::parse(&bibfile).unwrap();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
