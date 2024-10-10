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

use crate::frontend::app::App;
use crossterm::{
    cursor,
    event::{
        DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, MouseEvent,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
// use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::backend::CrosstermBackend as Backend;
use std::io::{stdout, Stdout};
use std::panic;
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use color_eyre::config::HookBuilder;
use color_eyre::eyre::{OptionExt, Result};
use futures::{FutureExt, StreamExt};
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

// pub type IO = std::io::{{crossterm_io | title_case}};
// pub fn io() -> IO {
//   std::io::{{crossterm_io}}()
// }
/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui {
    /// Interface to the Terminal.
    pub terminal: ratatui::Terminal<Backend<Stdout>>,
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
    /// Event handler thread.
    handler: tokio::task::JoinHandle<()>,
    cancellation_token: CancellationToken,
}

impl Tui {
    /// Constructs a new instance of [`Tui`].
    pub fn new() -> Result<Self> {
        let terminal = ratatui::Terminal::new(Backend::new(stdout()))?;
        let (sender, receiver) = mpsc::unbounded_channel();
        let handler = tokio::spawn(async {});
        let cancellation_token = CancellationToken::new();
        Ok(Self {
            terminal,
            sender,
            receiver,
            handler,
            cancellation_token,
        })
    }

    pub fn start(&mut self) {
        let tick_rate = Duration::from_millis(1000);
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let event_loop = Self::event_loop(
            self.sender.clone(),
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

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    // pub fn init(&mut self) -> Result<()> {
    //     terminal::enable_raw_mode()?;
    //     crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    //     // Define a custom panic hook to reset the terminal properties.
    //     // This way, you won't have your terminal messed up if an unexpected error happens.
    //     let panic_hook = panic::take_hook();
    //     panic::set_hook(Box::new(move |panic| {
    //         Self::reset().expect("failed to reset the terminal");
    //         panic_hook(panic);
    //     }));

    //     self.terminal.hide_cursor()?;
    //     self.terminal.clear()?;
    //     Ok(())
    // }

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
            self.flush()?;
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

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: ratatui::Terminal::draw
    /// [`rendering`]: crate::ui::render
    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        // self.terminal.draw(|frame| ui::render(app, frame))?;
        self.terminal
            .draw(|frame| frame.render_widget(app, frame.area()))?;
        Ok(())
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.receiver.recv().await.ok_or_eyre("This is an IO error")
    }

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
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<Stdout>>;

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
