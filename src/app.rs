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

use crate::bibiman::CurrentArea;
// use super::Event;
use crate::cliargs::CLIArgs;
use crate::tui::commands::{InputCmdAction, OpenRessource};
use crate::tui::{self, Tui};
use crate::{bibiman::Bibiman, tui::commands::CmdAction};
use color_eyre::eyre::{Ok, Result};
use tui::Event;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

// Application.
#[derive(Debug)]
pub struct App {
    // Is the application running?
    pub running: bool,
    // bibimain
    pub bibiman: Bibiman,
    // Input mode
    pub input: Input,
    // Input mode bool
    pub input_mode: bool,
}

impl App {
    // Constructs a new instance of [`App`].
    pub fn new(args: CLIArgs) -> Result<Self> {
        // Self::default()
        let running = true;
        let input = Input::default();
        let bibiman = Bibiman::new(args)?;
        Ok(Self {
            running,
            bibiman,
            input,
            input_mode: false,
        })
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
                // Event::Key(key_event) => handle_key_events(key_event, self, &mut tui)?,
                // Event::Mouse(_) => {}
                Event::Key(key_event) => {
                    let command = if self.input_mode {
                        CmdAction::Input(InputCmdAction::parse(key_event, &self.input))
                    } else {
                        CmdAction::from(key_event)
                    };
                    self.run_command(command, &mut tui)?
                }
                Event::Mouse(mouse_event) => {
                    self.run_command(CmdAction::from(mouse_event), &mut tui)?
                }

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

    pub fn run_command(&mut self, cmd: CmdAction, tui: &mut Tui) -> Result<()> {
        match cmd {
            CmdAction::Input(cmd) => match cmd {
                InputCmdAction::Nothing => {}
                InputCmdAction::Handle(event) => {
                    self.input.handle_event(&event);
                    self.bibiman.search_list_by_pattern(&self.input);
                }
                InputCmdAction::Enter => {
                    self.input_mode = true;
                    // Logic for TABS to be added
                    self.bibiman.enter_search_area();
                }
                InputCmdAction::Confirm => {
                    self.input = Input::default();
                    self.input_mode = false;
                    // Logic for TABS to be added
                    self.bibiman.confirm_search();
                }
                InputCmdAction::Exit => {
                    self.input = Input::default();
                    self.input_mode = false;
                    self.bibiman.break_search();
                }
            },
            CmdAction::SelectNextRow(amount) => match self.bibiman.current_area {
                // Here add logic to select TAB
                CurrentArea::EntryArea => {
                    self.bibiman.select_next_entry(amount);
                }
                CurrentArea::TagArea => {
                    self.bibiman.select_next_tag(amount);
                }
                _ => {}
            },
            CmdAction::SelectPrevRow(amount) => match self.bibiman.current_area {
                // Here add logic to select TAB
                CurrentArea::EntryArea => {
                    self.bibiman.select_previous_entry(amount);
                }
                CurrentArea::TagArea => {
                    self.bibiman.select_previous_tag(amount);
                }
                _ => {}
            },
            CmdAction::SelectNextCol => {
                if let CurrentArea::EntryArea = self.bibiman.current_area {
                    self.bibiman.select_next_column();
                }
            }
            CmdAction::SelectPrevCol => {
                if let CurrentArea::EntryArea = self.bibiman.current_area {
                    self.bibiman.select_prev_column();
                }
            }
            CmdAction::ScrollInfoDown => {
                self.bibiman.scroll_info_down();
            }
            CmdAction::ScrollInfoUp => {
                self.bibiman.scroll_info_up();
            }
            CmdAction::Bottom => match self.bibiman.current_area {
                CurrentArea::EntryArea => {
                    self.bibiman.select_last_entry();
                }
                CurrentArea::TagArea => {
                    self.bibiman.select_last_tag();
                }
                _ => {}
            },
            CmdAction::Top => match self.bibiman.current_area {
                CurrentArea::EntryArea => {
                    self.bibiman.select_first_entry();
                }
                CurrentArea::TagArea => {
                    self.bibiman.select_first_tag();
                }
                _ => {}
            },
            CmdAction::ToggleArea => {
                self.bibiman.toggle_area();
            }
            CmdAction::SearchList => {}
            CmdAction::ResetList => {
                self.bibiman.reset_current_list();
            }
            CmdAction::Confirm => {
                if let CurrentArea::TagArea = self.bibiman.current_area {
                    self.bibiman.filter_for_tags();
                }
            }
            CmdAction::SortList => {
                if let CurrentArea::EntryArea = self.bibiman.current_area {
                    self.bibiman.entry_table.sort_entry_table(true);
                }
            }
            CmdAction::YankItem => {
                Bibiman::yank_text(&self.bibiman.get_selected_citekey());
            }
            CmdAction::EditFile => {
                if let CurrentArea::EntryArea = self.bibiman.current_area {
                    self.bibiman.run_editor(tui)?;
                }
            }
            CmdAction::Open(ressource) => match ressource {
                OpenRessource::Pdf => {
                    if let CurrentArea::EntryArea = self.bibiman.current_area {
                        self.bibiman.open_connected_file()?;
                    }
                }
                OpenRessource::WebLink => {
                    if let CurrentArea::EntryArea = self.bibiman.current_area {
                        self.bibiman.open_doi_url()?;
                    }
                }
                OpenRessource::Note => {}
            },
            CmdAction::Exit => {
                self.quit();
            }
            CmdAction::Nothing => {}
        }
        Ok(())
    }
}
