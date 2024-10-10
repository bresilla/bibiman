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

use color_eyre::config::HookBuilder;
use color_eyre::eyre::Result;
use crossterm::cursor;
use crossterm::event::DisableMouseCapture;
use crossterm::terminal::LeaveAlternateScreen;
use std::io::stdout;

// Define error hooks to restore the terminal after panic
pub fn init_error_hooks() -> Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = crossterm::execute!(
            stdout(),
            DisableMouseCapture,
            LeaveAlternateScreen,
            cursor::Show
        );
        let _ = crossterm::terminal::disable_raw_mode();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::execute!(
            stdout(),
            DisableMouseCapture,
            LeaveAlternateScreen,
            cursor::Show
        );
        let _ = crossterm::terminal::disable_raw_mode();
        panic(info)
    }));
    Ok(())
}
