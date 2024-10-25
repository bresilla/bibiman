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

use crate::bibiman::entries::EntryTableColumn;
use crate::bibiman::keywords::TagListItem;
use crate::bibiman::{CurrentArea, FormerArea};
use crate::App;
use ratatui::layout::{Direction, Position};
use ratatui::widgets::Clear;
use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, HighlightSpacing, List, ListItem, Padding, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, Table, Wrap,
    },
};

const MAIN_BLUE_COLOR: Color = Color::Indexed(39);
// const MAIN_PURPLE_COLOR: Color = Color::Indexed(129);
const BOX_SELECTED_BOX_STYLE: Style = Style::new().fg(TEXT_FG_COLOR);
const BOX_SELECTED_TITLE_STYLE: Style = Style::new().fg(TEXT_FG_COLOR).add_modifier(Modifier::BOLD);
const BOX_UNSELECTED_BORDER_STYLE: Style = Style::new().fg(TEXT_UNSELECTED_FG_COLOR);
const BOX_UNSELECTED_TITLE_STYLE: Style = Style::new()
    .fg(TEXT_UNSELECTED_FG_COLOR)
    .add_modifier(Modifier::BOLD);
const NORMAL_ROW_BG: Color = Color::Black;
const ALT_ROW_BG_COLOR: Color = Color::Indexed(234);
const SELECTED_ROW_STYLE: Style = Style::new()
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::REVERSED);
const SELECTED_TABLE_COL_STYLE: Style = Style::new()
    // .add_modifier(Modifier::BOLD)
    .fg(Color::Indexed(254));
const SELECTEC_TABLE_CELL_STYLE: Style = Style::new()
    // .bg(Color::Indexed(250))
    .add_modifier(Modifier::REVERSED);
const TEXT_FG_COLOR: Color = Color::Indexed(250);
const TEXT_HIGHLIGHTED_COLOR: Color = Color::Indexed(254);
const TEXT_UNSELECTED_FG_COLOR: Color = Color::Indexed(245);
const SORTED_ENTRIES: &str = "▼";
const SORTED_ENTRIES_REVERSED: &str = "▲";
const HEADER_FOOTER_BG: Color = Color::Indexed(235);

const SCROLLBAR_UPPER_CORNER: Option<&str> = Some("┓");
const SCROLLBAR_LOWER_CORNER: Option<&str> = Some("┛");

pub const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl From<&TagListItem> for ListItem<'_> {
    fn from(value: &TagListItem) -> Self {
        let line = Line::styled(format!("{}", value.keyword), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}

pub fn render_ui(app: &mut App, frame: &mut Frame) {
    let [header_area, main_area, footer_area] = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
        ],
    )
    .direction(Direction::Vertical)
    .areas(frame.area());

    let [list_area, item_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

    let [entry_area, entry_info_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(list_area);

    let [tag_area, info_area] =
        Layout::horizontal([Constraint::Max(25), Constraint::Min(35)]).areas(item_area);

    render_header(frame, header_area);
    render_footer(app, frame, footer_area);
    render_file_info(app, frame, entry_info_area);
    render_entrytable(app, frame, entry_area);
    render_selected_item(app, frame, info_area);
    render_taglist(app, frame, tag_area);
}

pub fn render_header(frame: &mut Frame, rect: Rect) {
    let main_header = Paragraph::new("BIBIMAN – BibLaTeX manager TUI")
        .bold()
        .fg(MAIN_BLUE_COLOR)
        .centered();
    frame.render_widget(main_header, rect)
}

pub fn render_footer(app: &mut App, frame: &mut Frame, rect: Rect) {
    match &app.bibiman.current_area {
        CurrentArea::SearchArea => {
            let search_title = {
                match app.bibiman.former_area {
                    Some(FormerArea::EntryArea) => {
                        let search_title = " Search Entries ".to_string();
                        search_title
                    }
                    Some(FormerArea::TagArea) => {
                        let search_title = " Search Keywords ".to_string();
                        search_title
                    }
                    _ => {
                        let search_title = " Search ".to_string();
                        search_title
                    }
                }
            };

            let block = Block::bordered()
                .title(Line::styled(search_title, BOX_SELECTED_TITLE_STYLE))
                .border_style(BOX_SELECTED_BOX_STYLE)
                .border_set(symbols::border::THICK);
            render_cursor(app, frame, rect);
            frame.render_widget(
                Paragraph::new(app.bibiman.search_struct.search_string.clone())
                    .block(block)
                    .fg(TEXT_FG_COLOR),
                rect,
            );
        }
        _ => {
            let style_emph = Style::new().bold().fg(TEXT_FG_COLOR);
            let block = Block::bordered()
                .title(Line::raw(" Basic Commands ").centered())
                .border_style(BOX_UNSELECTED_BORDER_STYLE)
                .border_set(symbols::border::PLAIN);
            let keybindigns = Paragraph::new(Line::from(vec![
                Span::styled("j/k: ", style_emph),
                Span::raw("move | "),
                Span::styled("g/G: ", style_emph),
                Span::raw("top/bottom | "),
                Span::styled("TAB: ", style_emph),
                Span::raw("switch tab | "),
                Span::styled("y: ", style_emph),
                Span::raw("yank citekey | "),
                Span::styled("e: ", style_emph),
                Span::raw("edit | "),
                Span::styled("/: ", style_emph),
                Span::raw("search | "),
                Span::styled("o/u: ", style_emph),
                Span::raw("open PDF/DOI"),
            ]))
            .block(block)
            .centered();
            frame.render_widget(keybindigns, rect);
        }
    }
}

// Render info of the current file and process
// 1. Basename of the currently loaded file
// 2. Keyword by which the entries are filtered at the moment
// 3. Currently selected entry and total count of entries
pub fn render_file_info(app: &mut App, frame: &mut Frame, rect: Rect) {
    let block = Block::new() // can also be Block::new
        // Leave Top empty to simulate one large box with borders of entry list
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_set(if let CurrentArea::EntryArea = app.bibiman.current_area {
            symbols::border::THICK
        } else {
            symbols::border::PLAIN
        })
        .border_style(if let CurrentArea::EntryArea = app.bibiman.current_area {
            BOX_SELECTED_BOX_STYLE
        } else {
            BOX_UNSELECTED_BORDER_STYLE
        });

    frame.render_widget(block, rect);

    let [file_area, keyword_area, count_area] = Layout::horizontal([
        Constraint::Fill(3),
        Constraint::Fill(4),
        Constraint::Fill(1),
    ])
    .horizontal_margin(1)
    .areas(rect);

    let file_info = Line::from(vec![
        Span::raw("File: ").bold(),
        Span::raw(
            app.bibiman
                .main_bibfile
                .file_name()
                .unwrap()
                .to_string_lossy(),
        )
        .bold(),
    ])
    .bg(HEADER_FOOTER_BG);
    // .render(file_area, buf);

    let cur_keywords = Line::from(if !app.bibiman.tag_list.selected_keywords.is_empty() {
        vec![
            Span::raw("Selected keywords: "),
            // Show all keywords in correct order if list is filtered
            // successively by multiple keywords
            Span::raw(app.bibiman.tag_list.selected_keywords.join(" → "))
                .bold()
                .green(),
        ]
    } else {
        vec![Span::raw(" ")]
    })
    .bg(HEADER_FOOTER_BG);
    // .render(keyword_area, buf);

    let item_count = Line::from(
        if app
            .bibiman
            .entry_table
            .entry_table_state
            .selected()
            .is_some()
        {
            vec![
                Span::raw(
                    // Because method scroll_down_by() of TableState lets numbers
                    // printed overflow for short moment, we have to check manually
                    // that we do not print a number higher than len() of table
                    if app
                        .bibiman
                        .entry_table
                        .entry_table_state
                        .selected()
                        .unwrap()
                        + 1
                        > app.bibiman.entry_table.entry_table_items.len()
                    {
                        app.bibiman.entry_table.entry_table_items.len().to_string()
                    } else {
                        (app.bibiman
                            .entry_table
                            .entry_table_state
                            .selected()
                            .unwrap()
                            + 1)
                        .to_string()
                    },
                )
                .bold(),
                Span::raw("/"),
                Span::raw(app.bibiman.entry_table.entry_table_items.len().to_string()),
            ]
        } else {
            vec![Span::raw("No entries")]
        },
    )
    .right_aligned()
    .bg(HEADER_FOOTER_BG);
    frame.render_widget(file_info, file_area);
    frame.render_widget(cur_keywords, keyword_area);
    frame.render_widget(item_count, count_area);
}

pub fn render_entrytable(app: &mut App, frame: &mut Frame, rect: Rect) {
    let block = Block::new() // can also be Block::new
        .title(
            Line::styled(
                " Bibliographic Entries ",
                if let CurrentArea::EntryArea = app.bibiman.current_area {
                    BOX_SELECTED_TITLE_STYLE
                } else {
                    BOX_UNSELECTED_TITLE_STYLE
                },
            )
            .centered(),
        )
        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
        .border_set(if let CurrentArea::EntryArea = app.bibiman.current_area {
            symbols::border::THICK
        } else {
            symbols::border::PLAIN
        })
        .border_style(if let CurrentArea::EntryArea = app.bibiman.current_area {
            BOX_SELECTED_BOX_STYLE
        } else {
            BOX_UNSELECTED_BORDER_STYLE
        });

    let header_style = Style::default()
        .bold()
        .fg(TEXT_FG_COLOR)
        .bg(HEADER_FOOTER_BG);

    let header = Row::new(vec![
        Cell::from(
            Line::from(vec![{ Span::raw("Author") }, {
                if let EntryTableColumn::Authors = app.bibiman.entry_table.entry_table_sorted_by_col
                {
                    Span::raw(format!(
                        " {}",
                        if app.bibiman.entry_table.entry_table_reversed_sort {
                            SORTED_ENTRIES_REVERSED
                        } else {
                            SORTED_ENTRIES
                        }
                    ))
                } else {
                    Span::raw("")
                }
            }])
            .bg(
                if let EntryTableColumn::Authors =
                    app.bibiman.entry_table.entry_table_selected_column
                {
                    Color::Indexed(237)
                } else {
                    HEADER_FOOTER_BG
                },
            ),
        ),
        Cell::from(
            Line::from(vec![{ Span::raw("Title") }, {
                if let EntryTableColumn::Title = app.bibiman.entry_table.entry_table_sorted_by_col {
                    Span::raw(format!(
                        " {}",
                        if app.bibiman.entry_table.entry_table_reversed_sort {
                            SORTED_ENTRIES_REVERSED
                        } else {
                            SORTED_ENTRIES
                        }
                    ))
                } else {
                    Span::raw("")
                }
            }])
            .bg(
                if let EntryTableColumn::Title = app.bibiman.entry_table.entry_table_selected_column
                {
                    Color::Indexed(237)
                } else {
                    HEADER_FOOTER_BG
                },
            ),
        ),
        Cell::from(
            Line::from(vec![{ Span::raw("Year") }, {
                if let EntryTableColumn::Year = app.bibiman.entry_table.entry_table_sorted_by_col {
                    Span::raw(format!(
                        " {}",
                        if app.bibiman.entry_table.entry_table_reversed_sort {
                            SORTED_ENTRIES_REVERSED
                        } else {
                            SORTED_ENTRIES
                        }
                    ))
                } else {
                    Span::raw("")
                }
            }])
            .bg(
                if let EntryTableColumn::Year = app.bibiman.entry_table.entry_table_selected_column
                {
                    Color::Indexed(237)
                } else {
                    HEADER_FOOTER_BG
                },
            ),
        ),
        Cell::from(
            Line::from(vec![{ Span::raw("Pubtype") }, {
                if let EntryTableColumn::Pubtype = app.bibiman.entry_table.entry_table_sorted_by_col
                {
                    Span::raw(format!(
                        " {}",
                        if app.bibiman.entry_table.entry_table_reversed_sort {
                            SORTED_ENTRIES_REVERSED
                        } else {
                            SORTED_ENTRIES
                        }
                    ))
                } else {
                    Span::raw("")
                }
            }])
            .bg(
                if let EntryTableColumn::Pubtype =
                    app.bibiman.entry_table.entry_table_selected_column
                {
                    Color::Indexed(237)
                } else {
                    HEADER_FOOTER_BG
                },
            ),
        ),
    ])
    .style(header_style)
    .height(1);

    // Iterate over vector storing each entries data fields
    let rows = app
        .bibiman
        .entry_table
        .entry_table_items
        .iter_mut()
        .enumerate()
        .map(|(_i, data)| {
            let item = data.ref_vec();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("{content}"))))
                .collect::<Row>()
                .style(Style::new().fg(TEXT_FG_COLOR)) //.bg(alternate_colors(i)))
                .height(1)
        });
    let entry_table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Fill(1),
            Constraint::Length(
                if let EntryTableColumn::Year = app.bibiman.entry_table.entry_table_sorted_by_col {
                    6
                } else {
                    4
                },
            ),
            Constraint::Percentage(10),
        ],
    )
    .block(block)
    .header(header)
    .column_spacing(2)
    .row_highlight_style(SELECTED_ROW_STYLE)
    .column_highlight_style(SELECTED_TABLE_COL_STYLE)
    .cell_highlight_style(SELECTEC_TABLE_CELL_STYLE)
    .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(
        entry_table,
        rect,
        &mut app.bibiman.entry_table.entry_table_state,
    );

    // Scrollbar for entry table
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .track_symbol(None)
        .begin_symbol(SCROLLBAR_UPPER_CORNER)
        .end_symbol(None)
        .thumb_style(Style::new().fg(Color::DarkGray));

    if let CurrentArea::EntryArea = app.bibiman.current_area {
        // render the scrollbar
        frame.render_stateful_widget(
            scrollbar,
            rect,
            &mut app.bibiman.entry_table.entry_scroll_state,
        );
    }
}

pub fn render_selected_item(app: &mut App, frame: &mut Frame, rect: Rect) {
    // We get the info depending on the item's state.
    let style_value = Style::new().bold().fg(TEXT_FG_COLOR);
    let style_value_sec = Style::new()
        .add_modifier(Modifier::ITALIC)
        .fg(TEXT_FG_COLOR);
    let lines = {
        if app
            .bibiman
            .entry_table
            .entry_table_state
            .selected()
            .is_some()
        {
            let idx = app
                .bibiman
                .entry_table
                .entry_table_state
                .selected()
                .unwrap();
            let cur_entry = &app.bibiman.entry_table.entry_table_items[idx];
            let mut lines = vec![];
            lines.push(Line::from(vec![
                Span::styled("Authors: ", style_value),
                // Span::styled(cur_entry.authors.clone(), Style::new().green()),
                Span::styled(cur_entry.authors(), Style::new().green()),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Title: ", style_value),
                Span::styled(cur_entry.title(), Style::new().magenta()),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Year: ", style_value),
                Span::styled(cur_entry.year(), Style::new().light_magenta()),
            ]));
            // Render keywords in info box in Markdown code style
            if !cur_entry.keywords.is_empty() {
                let kw: Vec<&str> = cur_entry
                    .keywords
                    .split(",")
                    .map(|k| k.trim())
                    .filter(|k| !k.is_empty())
                    .collect();
                let mut content = vec![Span::styled("Keywords: ", style_value)];
                for k in kw {
                    // Add half block highlighted in bg color to enlarge block
                    content.push(Span::raw("▐").fg(HEADER_FOOTER_BG));
                    content.push(Span::styled(
                        k,
                        Style::default().bg(HEADER_FOOTER_BG).fg(
                            // Highlight selected keyword green
                            if app
                                .bibiman
                                .tag_list
                                .selected_keywords
                                .iter()
                                .any(|e| e == k)
                            {
                                Color::Green
                            } else {
                                TEXT_FG_COLOR
                            },
                        ),
                    ));
                    content.push(Span::raw("▌").fg(HEADER_FOOTER_BG));
                }
                lines.push(Line::from(content))
            }
            if !cur_entry.doi_url.is_empty() || !cur_entry.filepath.is_empty() {
                lines.push(Line::raw(""));
            }
            if !cur_entry.doi_url.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("DOI/URL: ", style_value_sec),
                    Span::styled(
                        cur_entry.doi_url(),
                        Style::default().fg(TEXT_FG_COLOR).underlined(),
                    ),
                ]));
            }
            if !cur_entry.filepath.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("File: ", style_value_sec),
                    Span::styled(cur_entry.filepath(), Style::default().fg(TEXT_FG_COLOR)),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                cur_entry.abstract_text.clone(),
                Style::default().fg(TEXT_FG_COLOR),
            )]));
            lines
        } else {
            let lines = vec![
                Line::from(" "),
                Line::from("No entry selected".bold().into_centered_line().red()),
            ];
            lines
        }
    };
    let info = Text::from(lines);

    // We show the list item's info under the list in this paragraph
    let block = Block::bordered()
        .title(Line::raw(" Entry Information ").centered().bold())
        .border_set(symbols::border::PLAIN)
        .border_style(BOX_UNSELECTED_BORDER_STYLE)
        .padding(Padding::horizontal(1));

    // INFO: '.line_count' method only possible with unstable-rendered-line-info feature -> API might change: https://github.com/ratatui/ratatui/issues/293#ref-pullrequest-2027056434
    let box_height = Paragraph::new(info.clone())
        .block(block.clone())
        .wrap(Wrap { trim: false })
        .line_count(rect.width);
    // Make sure to allow scroll only if text is larger than the rendered area and stop scrolling when last line is reached
    let scroll_height = {
        if app.bibiman.entry_table.entry_info_scroll == 0 {
            app.bibiman.entry_table.entry_info_scroll
        } else if rect.height > box_height as u16 {
            app.bibiman.entry_table.entry_info_scroll = 0;
            app.bibiman.entry_table.entry_info_scroll
        } else if app.bibiman.entry_table.entry_info_scroll > (box_height as u16 + 2 - rect.height)
        {
            app.bibiman.entry_table.entry_info_scroll = box_height as u16 + 2 - rect.height;
            app.bibiman.entry_table.entry_info_scroll
        } else {
            app.bibiman.entry_table.entry_info_scroll
        }
    };

    // We can now render the item info
    let item_info = Paragraph::new(info)
        .block(
            block
                // Render arrows to show that info box has content outside the block
                .title_bottom(
                    Line::from(
                        if box_height > rect.height.into()
                            && app.bibiman.entry_table.entry_info_scroll
                                < box_height as u16 + 2 - rect.height
                        {
                            " ▼ "
                        } else {
                            ""
                        },
                    )
                    .alignment(Alignment::Right),
                )
                .title_top(
                    Line::from(if scroll_height > 0 { " ▲ " } else { "" })
                        .alignment(Alignment::Right),
                ),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll_height, 0));

    frame.render_widget(item_info, rect);
}

pub fn render_taglist(app: &mut App, frame: &mut Frame, rect: Rect) {
    let block = Block::bordered()
        .title(
            Line::styled(
                " Keywords ",
                if let CurrentArea::TagArea = app.bibiman.current_area {
                    BOX_SELECTED_TITLE_STYLE
                } else {
                    BOX_UNSELECTED_TITLE_STYLE
                },
            )
            .centered(),
        )
        .border_set(if let CurrentArea::TagArea = app.bibiman.current_area {
            symbols::border::THICK
        } else {
            symbols::border::PLAIN
        })
        .border_style(if let CurrentArea::TagArea = app.bibiman.current_area {
            BOX_SELECTED_BOX_STYLE
        } else {
            BOX_UNSELECTED_BORDER_STYLE
        });

    // Iterate through all elements in the `items` and stylize them.
    let items: Vec<ListItem> = app
        .bibiman
        .tag_list
        .tag_list_items
        .iter()
        .enumerate()
        .map(|(_i, todo_item)| {
            // let color = alternate_colors(i);
            ListItem::from(todo_item.to_owned()) //.bg(color)
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let list = List::new(items)
        .block(block)
        .highlight_style(SELECTED_ROW_STYLE)
        .fg(if let CurrentArea::TagArea = app.bibiman.current_area {
            TEXT_HIGHLIGHTED_COLOR
        } else {
            TEXT_FG_COLOR
        })
        // .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    // Save list length for calculating scrollbar need
    // Add 2 to compmensate lines of the block border
    let list_length = list.len() + 2;

    frame.render_stateful_widget(list, rect, &mut app.bibiman.tag_list.tag_list_state);

    // Scrollbar for keyword list
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .track_symbol(None)
        .begin_symbol(SCROLLBAR_UPPER_CORNER)
        .end_symbol(SCROLLBAR_LOWER_CORNER)
        .thumb_style(Style::new().fg(Color::DarkGray));

    if list_length > rect.height.into() {
        if let CurrentArea::TagArea = app.bibiman.current_area {
            // render the scrollbar
            frame.render_stateful_widget(
                scrollbar,
                rect,
                &mut app.bibiman.tag_list.tag_scroll_state,
            );
        }
    }
}

/// Render the cursor when in InputMode
fn render_cursor(app: &mut App, frame: &mut Frame, area: Rect) {
    let scroll = app.input.visual_scroll(area.width as usize);
    if app.input_mode {
        let (x, y) = (
            area.x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            area.bottom().saturating_sub(2),
        );
        frame.render_widget(
            Clear,
            Rect {
                x,
                y,
                width: 1,
                height: 1,
            },
        );
        frame.set_cursor_position(Position::new(x, y));
    }
}
