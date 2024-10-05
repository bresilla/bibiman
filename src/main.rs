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

use backend::cliargs::{self, CLIArgs};
use color_eyre::eyre::Result;
use frontend::app::App;

pub mod backend;
pub mod frontend;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let parsed_args = CLIArgs::new();

    // Print help if -h/--help flag is passed and exit
    if parsed_args.helparg {
        println!("{}", cliargs::help_func());
        std::process::exit(0);
    }

    // Print version if -v/--version flag is passed and exit
    if parsed_args.versionarg {
        println!("{}", cliargs::version_func());
        std::process::exit(0);
    }

    // Create an application.
    let mut app = App::new(parsed_args)?;

    app.run().await?;
    Ok(())
}
