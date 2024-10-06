# Bibiman

`bibiman` is a simple terminal user interface for handling your BibLaTeX database as part of a terminal-based scientific workflow.

Here's a small impression how it looks and works:

![A demo gif of bibiman](./img/bibiman-demo-vhs.gif) 

## Installation

For now, `bibiman` is only available via Codeberg. You have to build it from source yourself using `cargo` and `rustup`:

```bash
git clone https://codeberg.org/lukeflo/bibiman
cd bibiman

# Build the binary to /target/release
cargo build --release

# OR
# Install the binary to CARGO_HOME which normally is in PATH
cargo install --path=. --locked

# optional: create symlink:
ln -sf /target/release/bibiman ~/.local/bin

```

If you use the symlink option, you have to make sure that the directory containing the symlink is in your `PATH`.

## Usage

You need to pass a single `.bib` file as first positional argument:

`bibiman /path/to/bibfile.bib`

Of course, this can be aliased if you only use one main file. E.g. in `.bashrc`/`.zshrc`:

`alias bibi=bibiman /path/to/bibfile.bib`

## Features

For now, `bibiman` only has some very basic features implemented which are important for my personal workflow:

+ **Browse** through the bib entries using *Vim-like keybindings* and a *fuzzy search* mode.
+ **Filter** the bib entries by *keywords* (and afterwards filter further by fuzzy searching).
+ **Edit** the current entry by opening a *terminal-based editor* at the specific line
+ **Yank/Copy** the citekey of the current entry to the system clipboard.

Furthermore, I want to implement the following features:

- [ ] **Open PDF**: Open related PDF file (`file` BibLaTeX key) with keypress.
- [ ] **Open URL/DOI**: Open related DOI/URL with keypress.
- [ ] **Add Entry via DOI**: Download bibliographic metadata for inserted DOI and add it to `.bib` file

## Keybindings

Use the following keybindings to manage the TUI:

| Key   | Action    |
|--------------- | --------------- |
| **j/k** \| **Down/Up**   | Move selected list   |
| **g/G**   | Go to first/last entry   |
| **y**   | Yank/copy citekey of selected entry   |
| **e**   | Open editor at selected entry   |
| **TAB** | Switch between entries and keywords |
| **/** \| **Ctrl+f** | Enter search mode |
| **Enter** | Filter by selected keyword / Confirm search |
| **ESC** | Abort search / Reset current list |
| **q** \| **Ctrl-C** | Quit TUI |

## Search

The search mode uses the `nucleo-matcher` crate. Thus, *fuzzy searching* is enabled by default. You can use some special chars to alter pattern matching:

+ `^...` matches only the beginning of the string.
+ `...$` matches only the end of the string.
+ `'...` matches literally (no fuzzy matching).

## Edit bib entry

For now, the TUI only supports editors set through the environment variables `VISUAL` and `EDITOR` in this order. The fallback solution is `vi`.

I've tested the following editors (set as value of `VISUAL`):

- [X] **Helix**: `export VISUAL="hx"`
- [X] **Vim/Neovim**: `export VISUAL="vim/nvim"`
- [X] **Emacs (Terminal)**: `export VISUAL="emacs -nw"`
- [X] **Nano**: `export VISUAL="nano"`
- [X] **Emacs (GUI)**: `export VISUAL="emacs"` (open emacs in separate window, blocks the terminal running `bibiman` as long as emacs is opened)

Feel free to try other editors. Important is that the editor supports the argument `+..` to set the line number that the cursor should be placed at. Otherwise, the functionality might not work properly.

Since this behaviour is most likely supported on UNIX-based systems (Linux, MacOS) it might not work under Windows. I can't test it on a Windows machine, thus, there might be unexpected errors with it.

## Issues and code improvement

This is my first Rust project and, thus, also a learning process. If you find any issues or code flaws, please open an issue. I plan to make PRs possible in the future when its a little bit less early alpha state.

## Alternatives

`bibiman` is a project tailored to my personal needs. I use a single main file for all my bib entries and want to use `bibiman` mainly as kind of (terminal)-graphical wrapper for often emerging tasks, since I work in the terminal most of the time.

I used `JabRef` for many years, but its way to bloated in my eyes. There exists a bunch of other graphical tools. Just check it yourself...

But there are also some TUI alternatives with slightly different approaches. Maybe one of these might fit *your* personal needs better:

+ [bibman (Haskell)](https://codeberg.org/KMIJPH/bibman): A very nice CLI program including a TUI I also used for some times. It has way more features (export etc.) at the moment. The main difference is that its based on a multi file approach. If you also use a separate file per entry, look there!
+ [bibman (Python)](https://github.com/ductri/bibman): A TUI written in Python with focus on Zotero-like functions. If you're uses to Zotero, this might be a good fit.
+ [bibman (Perl)](https://github.com/maciejjan/bibman): A fast and simple TUI written in good ol' Perl. It looks like back in the days, but seems not being maintained anymore 
