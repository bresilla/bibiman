# Bibiman

<!-- [![noMSgithub badge](https://nogithub.codeberg.page/badge.svg)](https://nogithub.codeberg.page/) -->

`bibiman` is a simple terminal user interface for handling your BibLaTeX
database as part of a terminal-based scientific workflow.

Here's a small impression how it looks and works:

[![bibiman.gif](https://i.postimg.cc/c4mcVk4n/bibiman.gif)](https://postimg.cc/56Xvz3z4)

## Installation

You can install `bibiman` directly from `crates.io` using `cargo`:

```bash
cargo install bibiman
```

To use the version including the newest commits, you can clone the repo and
build it from source using `cargo`:

```bash
git clone https://codeberg.org/lukeflo/bibiman
cd bibiman

# Build the binary to /target/release
cargo build --release

# OR
# Install the binary to CARGO_HOME/bin which normally is in PATH
cargo install --path=. --locked

```

## Usage

The following arguments are possible:

```
USAGE:
    bibiman [FLAGS] [files/dirs]

POSITIONAL ARGS:
    <file>    Path to .bib file
    <dir>     Path to directory containing .bib files

FLAGS:
    -h, --help          Show this help and exit
    -v, --version       Show the version and exit
    --light-terminal    Enable color mode for light terminal background

```

As seen, you can pass a single file, multiple files, the path of a directory
containing bibfiles, or mix files and directories.

Directories will be searched recursively for files with the `.bib` extension and
add them to the entry list. Other files will be ignored.Thus, be careful not to
pass a directory with multiple subdirectories (like eg `/home/usr/`), because
this could lead to some delay while parsing GBs of data.

The following lines are all valid CLI calls to run `bibiman` using the test
files from the `tests` folder:

```bash
# single file
bibiman tests/biblatex-test.bib

# multiple files
bibiman tests/multi-files/bibfile1.bib tests/multi-files/bibfile2.bib

# directory containing bibfiles
bibman tests/multi-files/

# mixed arguments
bibiman tests/biblatex-test.bib tests/multi-files/
```

Here as small demo gif:

[![bibiman-multifile.gif](https://i.postimg.cc/pXR13Fxp/bibiman-multifile.gif)](https://postimg.cc/DmpgXmST)

## Features

For now, `bibiman` only has mainly features implemented which are important for
my personal workflow. There are more to come, the list will be updated:

- [x] **Browse** through the bib entries using _Vim-like keybindings_ and a
      _fuzzy search_ mode.
- [x] **Filter** the bib entries by _keywords_ (and afterwards filter further by
      fuzzy searching).
- [x] **Edit** the current entry by opening a _terminal-based editor_ at the
      specific line.
- [x] **Yank/Copy** the citekey of the current entry to the system clipboard.
- [x] **Open related PDF** file (`file` BibLaTeX key) with keypress.
- [x] **Open related URL/DOI** with keypress.
- [x] **Scrollbar** for better navigating.
- [x] **Sort Entries** by column (`Authors`, `Title`, `Year`, `Pubtype`)
- [x] **Load multiple files** into one session.
- [ ] **Open related notes file** for specific entry.
- [ ] **Add Entry via DOI** as formatted code.
- [ ] **Implement config file** for setting some default values like main
      bibfile, PDF-opener, or editor
- [ ] **Support Hayagriva(`.yaml`)** format as input (_on hold for now_, because
      the Hayagriva Yaml style doesn't offer keywords; s. issue in
      [Hayagriva repo](https://github.com/typst/hayagriva/issues/240)).

**Please feel free to suggest further features through the issue
functionality.**

## Keybindings

Use the following keybindings to manage the TUI:

| Key                                      | Action                                                   |
| ---------------------------------------- | -------------------------------------------------------- |
| `?`                                      | Open help popup with keybindings                         |
| `j`, `k` \| `Down`, `Up`                 | Move down/up by 1                                        |
| `Ctrl-d`, `Ctrl-u`                       | Move down/up by 5                                        |
| `g`, `G`                                 | Go to first/last entry                                   |
| `h`, `k` \| `Left`, `Right`              | Select previous/next entry column                        |
| `s`                                      | Sort current column (toggles)                            |
| `PageDown`, `PageUp` \| `Alt-j`, `Alt-k` | Scroll Info window                                       |
| `y`                                      | Yank/copy citekey of selected entry                      |
| `e`                                      | Open editor at selected entry                            |
| `o`                                      | Open related PDF or URL/DOI                              |
| `TAB`                                    | Switch between entries and keywords                      |
| `/`, `Ctrl-f`                            | Enter search mode                                        |
| `Enter`                                  | Filter by selected keyword / Confirm search or selection |
| `ESC`                                    | Abort search / Reset current list                        |
| `q`, `Ctrl-c`                            | Quit TUI                                                 |

## Search

The search mode uses the `nucleo-matcher` crate. Thus, _fuzzy searching_ is
enabled by default. You can use some special chars to alter pattern matching:

- `^...` matches literally at beginning of the string.
- `...$` matches literally at end of the string.
- `'...` matches literally everywhere in string.

## Edit bib entry

For now, the TUI only supports editors set through the environment variables
`VISUAL` and `EDITOR` in this order. The fallback solution is `vi`.

I've tested the following editors (set as value of `VISUAL`):

- [x] **Helix**: `export VISUAL="hx"`
- [x] **Vim/Neovim**: `export VISUAL="vim/nvim"`
- [x] **Emacs (Terminal)**: `export VISUAL="emacs -nw"`
- [x] **Nano**: `export VISUAL="nano"`
- [x] **Emacs (GUI)**: `export VISUAL="emacs"` (open emacs in separate window,
      blocks the terminal running `bibiman` as long as emacs is opened)

Feel free to try other editors and report. Important is that the editor supports
the argument `+..` to set the line number that the cursor should be placed at.
Otherwise, the functionality might not work properly.

While this behaviour is most likely supported on UNIX-based systems (Linux,
MacOS), it might not work under Windows. I can't test it on a Windows machine,
thus, there might be unexpected errors with it.

## Open connected files or links

`bibiman` also provides the possibility to open PDFs (as value of the `file`
BibLaTeX field), as well as DOIs and URLs.

For selecting the right program, it uses `xdg-open` on Linux, `open` on MacOS,
and `start` on Windows. Thanks to the report from @bastislack in #2 MacOS seems
to work.

_However, Windows does not work. Have to figure this out. Reports from some
Windows users are very welcome._

Furthermore, DOIs have to begin with either `https://doi...` as full URL or
`10.(...)` as regular DOI style. URLs work if they begin with either `http...`
or with `www...`.

## Issues and code improvement

This is my first Rust project and, thus, also a learning process. If you find
any issues or code flaws, please open an issue.

## Alternatives

`bibiman` is a project tailored to my personal needs. I use a single main file
for all my bib entries and want to use `bibiman` mainly as kind of
(terminal)-graphical wrapper for often emerging tasks, since I work in the
terminal most of the time.

I used `JabRef` for many years, but its way to bloated in my eyes. There exists
a bunch of other graphical tools...

But there are also some TUI alternatives with slightly different approaches.
Maybe one of these might fit _your_ personal needs better:

- [bibman (Haskell)](https://codeberg.org/KMIJPH/bibman): A very nice CLI
  program including a TUI I also used for some times. It has way more CLI
  features (export etc.) at the moment.
- [bibman (Python)](https://github.com/ductri/bibman): A TUI written in Python
  with focus on Zotero-like functions. If you're used to Zotero, this might be a
  good fit.
- [bibman (Perl)](https://github.com/maciejjan/bibman): A fast and simple TUI
  written in good ol' Perl. It looks like back in the days, but seems not being
  maintained anymore.
- [cobib](https://github.com/mrossinek/cobib): Very elaborated bib manager with
  CLI and TUI functions.
- [papis](https://github.com/papis/papis): Powerful CLI tool for managing
  bibliographies and documents. Has also some TUI features.

### Comparison

I compared `bibiman` only free-hand to `bibman` (Haskell) and `bibman` (Perl),
since there is no simple benchmark test for TUIs. At least, I couldn't find one.

Loading a test file containing 25.000 dummy entries as well as a directory
containing 25.000 single dummy `.bib` files `bibiman` was significantly faster
on startup than both other programs. The performance also did not suffer. Only
on a test file with more than 50.000 dummy entries a very short delay after
keypresses was recognizable when scrolling the entry list. This is very likely
due to the effect of the single entries being dimmed differently regarding their
position to the selected one which needs some CPU resources.

After all, `bibiman` is really fast and runs very smooth while having the most
complex user interface by far compared to the other programs.
