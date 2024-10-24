// bibiman - a TUI for managing BibLaTeX databases
// Copyright (C) 2024  lukeflo
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
/////

// use super::Event;
use crate::bibiman::Bibiman;
use crate::cliargs::CLIArgs;
use crate::tui;
use crate::tui::handler::handle_key_events;
use color_eyre::eyre::{Ok, Result};
use tui::Event;

// Application.
#[derive(Debug)]
pub struct App {
    // Is the application running?
    pub running: bool,
    // bibimain
    pub bibiman: Bibiman,
}

impl App {
    // Constructs a new instance of [`App`].
    pub fn new(args: CLIArgs) -> Result<Self> {
        // Self::default()
        let running = true;
        let bibiman = Bibiman::new(args)?;
        Ok(Self { running, bibiman })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = tui::Tui::new()?;
        tui.enter()?;

        // Start the main loop.
        while self.running {
            // Render the user interface.
            tui.draw(self)?;
            // Handle events.
            match tui.next().await? {
                Event::Tick => self.tick(),
                Event::Key(key_event) => handle_key_events(key_event, self, &mut tui)?,
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
            }
        }

        // Exit the user interface.
        tui.exit()?;
        Ok(())
    }

    // Handles the tick event of the terminal.
    pub fn tick(&self) {}

    // General commands

    // Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
