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

pub mod command;
pub mod commandnew;
pub mod handler;
pub mod ui;

use crate::App;
use crossterm::{
    cursor,
    event::{
        DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, MouseEvent,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
// use ratatui::backend::{Backend, CrosstermBackend};
use color_eyre::eyre::{OptionExt, Result};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend;
use std::io::{stdout, Stdout};
use std::panic;
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct Tui {
    /// Interface to the Terminal.
    pub terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
    /// Event sender channel.
    evt_sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    evt_receiver: mpsc::UnboundedReceiver<Event>,
    /// Event handler thread.
    handler: tokio::task::JoinHandle<()>,
    cancellation_token: CancellationToken,
}

impl Tui {
    // Constructs a new instance of [`Tui`].
    pub fn new() -> Result<Self> {
        let terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout()))?;
        let (evt_sender, evt_receiver) = mpsc::unbounded_channel();
        let handler = tokio::spawn(async {});
        let cancellation_token = CancellationToken::new();
        Ok(Self {
            terminal,
            evt_sender,
            evt_receiver,
            handler,
            cancellation_token,
        })
    }

    pub fn start(&mut self) {
        let tick_rate = Duration::from_millis(1000);
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let event_loop = Self::event_loop(
            self.evt_sender.clone(),
            self.cancellation_token.clone(),
            tick_rate,
        );
        // let _cancellation_token = self.cancellation_token.clone();
        // let _sender = self.sender.clone();
        self.handler = tokio::spawn(async {
            event_loop.await;
        });
    }

    async fn event_loop(
        sender: mpsc::UnboundedSender<Event>,
        cancellation_token: CancellationToken,
        tick_rate: Duration,
    ) {
        let mut reader = crossterm::event::EventStream::new();
        let mut tick = tokio::time::interval(tick_rate);
        loop {
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
                // _ = sender.closed() => {
                //   break;
                // }
                _ = cancellation_token.cancelled() => {
                  break;
                }
                Some(Ok(evt)) = crossterm_event => {
                    match evt {
                        CrosstermEvent::Key(key) => {
                            if key.kind == crossterm::event::KeyEventKind::Press {
                                sender.send(Event::Key(key)).unwrap();
                            }
                        },
                        CrosstermEvent::Mouse(mouse) => {
                            sender.send(Event::Mouse(mouse)).unwrap();
                        },
                        CrosstermEvent::Resize(x, y) => {
                            sender.send(Event::Resize(x, y)).unwrap();
                        },
                        CrosstermEvent::FocusLost => {
                        },
                        CrosstermEvent::FocusGained => {
                        },
                        CrosstermEvent::Paste(_) => {
                        },
                    }
                }
                _ = tick_delay => {
                    sender.send(Event::Tick).unwrap();
                }
            };
        }
        cancellation_token.cancel();
    }

    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        // if self.mouse {
        crossterm::execute!(stdout(), EnableMouseCapture)?;
        // }
        // if self.paste {
        //     crossterm::execute!(stdout(), EnableBracketedPaste)?;
        // }
        // Self::init_error_hooks()?;
        self.start();
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.cancellation_token.cancel();
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.terminal.flush()?;
            // if self.paste {
            //     crossterm::execute!(stdout(), DisableBracketedPaste)?;
            // }
            // if self.mouse {
            crossterm::execute!(stdout(), DisableMouseCapture)?;
            // }
            crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    // [`Draw`] the terminal interface by [`rendering`] the widgets.
    //
    // [`Draw`]: ratatui::Terminal::draw
    // [`rendering`]: crate::ui::render
    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        // self.terminal.draw(|frame| ui::render(app, frame))?;
        self.terminal
            // .draw(|frame| frame.render_widget(app, frame.area()))?;
            .draw(|frame| ui::render_ui(app, frame))?;
        Ok(())
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.evt_receiver
            .recv()
            .await
            .ok_or_eyre("This is an IO error")
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
