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

use crate::bibiman::{CurrentArea, FormerArea};
use color_eyre::eyre::{Context, Ok, Result};
// use super::Event;
use crate::cliargs::CLIArgs;
use crate::tui::commands::InputCmdAction;
use crate::tui::popup::PopupKind;
use crate::tui::{self, Tui};
use crate::{bibiman::Bibiman, tui::commands::CmdAction};
use ratatui::crossterm::event::KeyCode;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
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
    pub fn new(args: &CLIArgs) -> Result<Self> {
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

    pub async fn run(&mut self, args: &CLIArgs) -> Result<()> {
        let mut tui = tui::Tui::new()?;
        tui.enter()?;

        // Start the main loop.
        while self.running {
            // Render the user interface.
            tui.draw(self, args)?;
            // Handle events.
            match tui.next().await? {
                Event::Tick => self.tick(),
                // Event::Key(key_event) => handle_key_events(key_event, self, &mut tui)?,
                // Event::Mouse(_) => {}
                Event::Key(key_event) => {
                    // Automatically close message popups on next keypress
                    if let Some(PopupKind::MessageConfirm) = self.bibiman.popup_area.popup_kind {
                        self.bibiman.close_popup()
                    } else if let Some(PopupKind::MessageError) = self.bibiman.popup_area.popup_kind
                    {
                        self.bibiman.close_popup()
                    } else if let Some(PopupKind::AddEntry) = self.bibiman.popup_area.popup_kind {
                        // Handle key events for AddEntry popup
                        match key_event.code {
                            KeyCode::Char(c) => {
                                let index = self.bibiman.popup_area.add_entry_cursor_position;
                                self.bibiman.popup_area.add_entry_input.insert(index, c);
                                self.bibiman.popup_area.add_entry_cursor_position += 1;
                            }
                            KeyCode::Backspace => {
                                if self.bibiman.popup_area.add_entry_cursor_position > 0 {
                                    self.bibiman.popup_area.add_entry_cursor_position -= 1;
                                    let index = self.bibiman.popup_area.add_entry_cursor_position;
                                    self.bibiman.popup_area.add_entry_input.remove(index);
                                }
                            }
                            KeyCode::Left => {
                                if self.bibiman.popup_area.add_entry_cursor_position > 0 {
                                    self.bibiman.popup_area.add_entry_cursor_position -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if self.bibiman.popup_area.add_entry_cursor_position
                                    < self.bibiman.popup_area.add_entry_input.len()
                                {
                                    self.bibiman.popup_area.add_entry_cursor_position += 1;
                                }
                            }
                            KeyCode::Enter => {
                                // Handle submission of the new entry
                                self.bibiman.handle_new_entry_submission(args);
                                self.bibiman.close_popup();
                                self.input_mode = false;
                            }
                            KeyCode::Esc => {
                                // Close the popup without saving
                                self.bibiman.close_popup();
                                self.input_mode = false;
                            }
                            _ => {}
                        }
                    } else {
                        let command = if self.input_mode {
                            CmdAction::Input(InputCmdAction::parse(key_event, &self.input))
                        } else {
                            CmdAction::from(key_event)
                        };
                        self.run_command(command, args, &mut tui)?
                    }
                }
                // Event::Key(key_event) => {
                //     // Automatically close message popups on next keypress
                //     if let Some(PopupKind::MessageConfirm) = self.bibiman.popup_area.popup_kind {
                //         self.bibiman.close_popup()
                //     } else if let Some(PopupKind::MessageError) = self.bibiman.popup_area.popup_kind
                //     {
                //         self.bibiman.close_popup()
                //     }

                //     let command = if self.input_mode {
                //         CmdAction::Input(InputCmdAction::parse(key_event, &self.input))
                //     } else {
                //         CmdAction::from(key_event)
                //     };
                //     self.run_command(command, args, &mut tui)?
                // }
                Event::Mouse(mouse_event) => {
                    self.run_command(CmdAction::from(mouse_event), args, &mut tui)?
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

    pub fn run_command(&mut self, cmd: CmdAction, args: &CLIArgs, tui: &mut Tui) -> Result<()> {
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
                CurrentArea::PopupArea => {
                    if let Some(PopupKind::Help) = self.bibiman.popup_area.popup_kind {
                        self.bibiman.popup_area.popup_scroll_down();
                    } else if let Some(PopupKind::Selection) = self.bibiman.popup_area.popup_kind {
                        self.bibiman.popup_area.popup_state.scroll_down_by(1)
                    }
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
                CurrentArea::PopupArea => {
                    if let Some(PopupKind::Help) = self.bibiman.popup_area.popup_kind {
                        self.bibiman.popup_area.popup_scroll_up();
                    } else if let Some(PopupKind::Selection) = self.bibiman.popup_area.popup_kind {
                        self.bibiman.popup_area.popup_state.scroll_up_by(1)
                    }
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
            CmdAction::Reset => {
                if let CurrentArea::PopupArea = self.bibiman.current_area {
                    if let Some(PopupKind::Help) = self.bibiman.popup_area.popup_kind {
                        self.bibiman.popup_area.popup_scroll_pos = 0;
                        self.bibiman.close_popup()
                    } else if let Some(PopupKind::Selection) = self.bibiman.popup_area.popup_kind {
                        self.bibiman.close_popup()
                    }
                } else {
                    self.bibiman.reset_current_list();
                }
            }
            CmdAction::Confirm => {
                if let CurrentArea::TagArea = self.bibiman.current_area {
                    self.bibiman.filter_for_tags();
                } else if let CurrentArea::PopupArea = self.bibiman.current_area {
                    if let Some(PopupKind::Help) = self.bibiman.popup_area.popup_kind {
                        self.bibiman.close_popup();
                    } else if let Some(PopupKind::Selection) = self.bibiman.popup_area.popup_kind {
                        // Index of selected entry
                        let entry_idx = self
                            .bibiman
                            .entry_table
                            .entry_table_state
                            .selected()
                            .unwrap();

                        // Index of selected popup field
                        let popup_idx = self.bibiman.popup_area.popup_state.selected().unwrap();

                        // Choose ressource depending an selected popup field
                        if self.bibiman.popup_area.popup_list[popup_idx].contains("Weblink") {
                            let object =
                                self.bibiman.entry_table.entry_table_items[entry_idx].doi_url();
                            let url = prepare_weblink(object);
                            open_connected_link(&url)?;
                        } else if self.bibiman.popup_area.popup_list[popup_idx].contains("File") {
                            let object =
                                self.bibiman.entry_table.entry_table_items[entry_idx].filepath();
                            open_connected_file(object)?;
                        } else {
                            eprintln!("Unable to find ressource to open");
                        };
                        // run command to open file/Url
                        self.bibiman.close_popup()
                    }
                }
            }
            CmdAction::SortList => {
                if let CurrentArea::EntryArea = self.bibiman.current_area {
                    self.bibiman.entry_table.sort_entry_table(true);
                }
            }
            CmdAction::YankItem => {
                if let CurrentArea::EntryArea = self.bibiman.current_area {
                    let citekey: &str = &self.bibiman.entry_table.entry_table_items[self
                        .bibiman
                        .entry_table
                        .entry_table_state
                        .selected()
                        .unwrap()]
                    .citekey;

                    Bibiman::yank_text(citekey);
                    self.bibiman.popup_area.popup_message(
                        "Yanked citekey to clipboard: ",
                        citekey, // self.bibiman.get_selected_citekey(),
                        true,
                    );
                }
            }
            CmdAction::EditFile => {
                if let CurrentArea::EntryArea = self.bibiman.current_area {
                    self.bibiman.run_editor(args, tui)?;
                }
            }
            CmdAction::Open => {
                if let CurrentArea::EntryArea = self.bibiman.current_area {
                    let idx = self
                        .bibiman
                        .entry_table
                        .entry_table_state
                        .selected()
                        .unwrap();
                    let entry = self.bibiman.entry_table.entry_table_items[idx].clone();
                    if entry.filepath.is_some() || entry.doi_url.is_some() {
                        let mut items = vec![];
                        if entry.doi_url.is_some() {
                            items.push("Weblink (DOI/URL)".to_owned())
                        }
                        if entry.filepath.is_some() {
                            items.push("File (PDF/EPUB)".to_owned())
                        }
                        self.bibiman.popup_area.popup_selection(items);
                        self.bibiman.former_area = Some(FormerArea::EntryArea);
                        self.bibiman.current_area = CurrentArea::PopupArea;
                        self.bibiman.popup_area.popup_state.select(Some(0))
                    } else {
                        self.bibiman.popup_area.popup_message(
                            "Selected entry has no connected ressources: ",
                            &entry.citekey,
                            false,
                        )
                    }
                }
            }
            CmdAction::AddEntry => {
                self.bibiman.add_entry();
            }
            CmdAction::ShowHelp => {
                self.bibiman.show_help();
            }
            CmdAction::Exit => {
                self.quit();
            }
            CmdAction::Nothing => {}
        }
        Ok(())
    }
}

pub fn open_connected_file(file: &OsStr) -> Result<()> {
    // Build command to execute pdf-reader. 'xdg-open' is Linux standard
    // TODO: make custom opener command possible through config
    let cmd = {
        match std::env::consts::OS {
            "linux" => String::from("xdg-open"),
            "macos" => String::from("open"),
            "windows" => String::from("start"),
            _ => panic!("Couldn't detect OS for setting correct opener"),
        }
    };

    // If necessary, replace ~ with /home dir
    let file = PathBuf::from(file);

    let file = expand_home(&file);

    // Pass filepath as argument, pipe stdout and stderr to /dev/null
    // to keep the TUI clean (where is it piped on Windows???)
    let _ = Command::new(&cmd)
        .arg(file)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .wrap_err("Opening file not possible");

    Ok(())
}

pub fn open_connected_link(link: &str) -> Result<()> {
    // Build command to execute pdf-reader. 'xdg-open' is Linux standard
    // TODO: make custom opener command possible through config
    let cmd = {
        match std::env::consts::OS {
            "linux" => String::from("xdg-open"),
            "macos" => String::from("open"),
            "windows" => String::from("start"),
            _ => panic!("Couldn't detect OS for setting correct opener"),
        }
    };

    // Pass filepath as argument, pipe stdout and stderr to /dev/null
    // to keep the TUI clean (where is it piped on Windows???)
    let _ = Command::new(&cmd)
        .arg(link)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .wrap_err("Opening link not possible");

    Ok(())
}

pub fn prepare_weblink(url: &str) -> String {
    if url.starts_with("10.") {
        "https://doi.org/".to_string() + url
    } else if url.starts_with("www.") {
        "https://".to_string() + url
    } else {
        url.to_string()
    }
}

fn expand_home(path: &PathBuf) -> PathBuf {
    // let path = PathBuf::from(path);
    if path.starts_with("~") {
        let mut home = dirs::home_dir().unwrap();
        let path = path.strip_prefix("~").unwrap();
        home.push(path);
        home
    } else {
        path.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_home_expansion() {
        let path: PathBuf = "~/path/to/file.txt".into();

        let path = expand_home(&path);

        let home: String = dirs::home_dir().unwrap().to_str().unwrap().to_string();

        let full_path = home + "/path/to/file.txt";

        assert_eq!(path, PathBuf::from(full_path))
    }
}
