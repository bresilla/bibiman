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

use std::path::PathBuf;

use super::colors::AppColorScheme;
use super::popup::PopupArea;
use crate::bibiman::entries::EntryTableColumn;
use crate::bibiman::{CurrentArea, FormerArea};
use crate::cliargs::CLIArgs;
use crate::tui::popup::PopupKind;
use crate::App;
use ratatui::layout::{Direction, Position};
use ratatui::widgets::Clear;
use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, HighlightSpacing, List, ListItem, Padding, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, Table, Wrap,
    },
};
use walkdir::WalkDir;

// Symbols
static SORTED_ENTRIES: &str = "▼";
static SORTED_ENTRIES_REVERSED: &str = "▲";
static SCROLLBAR_UPPER_CORNER: Option<&str> = Some("┓");
static SCROLLBAR_LOWER_CORNER: Option<&str> = Some("┛");

pub fn color_list(
    args: &CLIArgs,
    list_item: i32,
    sel_item: i32,
    highlight: u8,
    max_diff: i32,
) -> Color {
    match args.colors.color_scheme {
        AppColorScheme::Dark => {
            if list_item == sel_item {
                Color::Indexed(highlight)
            } else if (list_item - sel_item) > max_diff
                || (sel_item - list_item) > max_diff
                || -(list_item - sel_item) > max_diff
                || -(sel_item - list_item) > max_diff
            {
                Color::Indexed(highlight - max_diff as u8)
            } else if list_item < sel_item {
                Color::Indexed(highlight - (sel_item - list_item) as u8)
            } else {
                Color::Indexed(highlight - (list_item - sel_item) as u8)
            }
        }
        AppColorScheme::Light => {
            if list_item == sel_item {
                Color::Indexed(highlight)
            } else if (list_item - sel_item) > max_diff
                || (sel_item - list_item) > max_diff
                || -(list_item - sel_item) > max_diff
                || -(sel_item - list_item) > max_diff
            {
                Color::Indexed(highlight + max_diff as u8)
            } else if list_item < sel_item {
                Color::Indexed(highlight + (sel_item - list_item) as u8)
            } else {
                Color::Indexed(highlight + (list_item - sel_item) as u8)
            }
        }
    }
}

fn count_files(files: &[PathBuf]) -> u16 {
    let mut count = 0;
    for f in files {
        if f.is_file() {
            count += 1
        } else if f.is_dir() {
            for e in WalkDir::new(f) {
                let f = e.unwrap().into_path();
                if f.is_file() && f.extension().unwrap() == "bib" {
                    count += 1
                }
            }
        }
    }
    count
}

pub fn render_ui(app: &mut App, args: &CLIArgs, frame: &mut Frame) {
    let [header_area, main_area, footer_area] = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(if let CurrentArea::SearchArea = app.bibiman.current_area {
                1
            } else {
                0
            }),
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

    render_header(args, frame, header_area);
    if let CurrentArea::SearchArea = app.bibiman.current_area {
        render_footer(app, args, frame, footer_area);
    }
    render_entrytable(app, args, frame, entry_area);
    render_selected_item(app, args, frame, info_area);
    render_taglist(app, args, frame, tag_area);
    render_file_info(app, args, frame, entry_info_area);
    if app.bibiman.popup_area.is_popup {
        render_popup(app, args, frame);
    }
}

pub fn render_popup(app: &mut App, args: &CLIArgs, frame: &mut Frame) {
    match app.bibiman.popup_area.popup_kind {
        Some(PopupKind::Help) => {
            let block = Block::bordered()
                .title_top(" Keybindings ".bold())
                .title_bottom(" (j,k|↓,↑) ━ (ESC|ENTER) ".bold())
                .title_alignment(Alignment::Center)
                .style(
                    Style::new()
                        .fg(Color::Indexed(args.colors.main_text_color))
                        .bg(Color::Indexed(args.colors.popup_bg_color)),
                )
                .border_set(symbols::border::THICK)
                .border_style(Style::new().fg(Color::Indexed(args.colors.entry_color)));

            let text: Text = PopupArea::popup_help(args);

            // Calculate max scroll position depending on hight of terminal window
            // Needed length is number of text lines plus two for borders at bottom and top
            // minus half the height of the frame (or the height set for the popup)
            let popup_height: u16 = frame.area().height / 2;

            let scroll_pos = if app.bibiman.popup_area.popup_scroll_pos
                > text.lines.len() as u16 + 2 - popup_height
            {
                app.bibiman.popup_area.popup_scroll_pos =
                    text.lines.len() as u16 + 2 - popup_height;
                app.bibiman.popup_area.popup_scroll_pos
            } else {
                app.bibiman.popup_area.popup_scroll_pos
            };

            let par = Paragraph::new(text).scroll((scroll_pos, 0)).block(block);
            let par_width = par.line_width();

            let popup_area = popup_area(frame.area(), par_width as u16, popup_height);

            frame.render_widget(Clear, popup_area);
            frame.render_widget(par, popup_area)
        }
        Some(PopupKind::MessageConfirm) => {
            let area = frame.area();

            let block = Block::bordered()
                .title_top(
                    " Message "
                        .bold()
                        .fg(Color::Indexed(args.colors.confirm_color)),
                )
                .border_style(Style::new().fg(Color::Indexed(args.colors.confirm_color)))
                .style(
                    Style::new()
                        .fg(Color::Indexed(args.colors.main_text_color))
                        .bg(Color::Indexed(args.colors.popup_bg_color)),
                );

            let content = Paragraph::new(app.bibiman.popup_area.popup_message.clone())
                .block(block)
                .style(Style::new().fg(Color::Indexed(args.colors.confirm_color)));

            // Calculate popup size. Width is number of string chars plus 2 for border
            let popup_area = popup_area(
                area,
                (app.bibiman.popup_area.popup_message.chars().count() + 2) as u16,
                3,
            );

            // Clear area and draw popup
            frame.render_widget(Clear, popup_area);
            frame.render_widget(&content, popup_area)
        }
        Some(PopupKind::MessageError) => {
            let area = frame.area();

            let block = Block::bordered()
                .title_top(
                    " Warning "
                        .bold()
                        .fg(Color::Indexed(args.colors.warn_color)),
                )
                .border_style(Style::new().fg(Color::Red))
                .style(
                    Style::new()
                        .fg(Color::Indexed(args.colors.main_text_color))
                        .bg(Color::Indexed(args.colors.popup_bg_color)),
                );

            let content = Paragraph::new(app.bibiman.popup_area.popup_message.clone())
                .block(block)
                .style(Style::new().fg(Color::Indexed(args.colors.warn_color)));

            // Calculate popup size. Width is number of string chars plus 2 for border
            let popup_area = popup_area(
                area,
                (app.bibiman.popup_area.popup_message.chars().count() + 2) as u16,
                3,
            );

            // Clear area and draw popup
            frame.render_widget(Clear, popup_area);
            frame.render_widget(&content, popup_area)
        }
        Some(PopupKind::Selection) => {
            let list_items: Vec<ListItem> = app
                .bibiman
                .popup_area
                .popup_list
                .iter()
                .map(|item| ListItem::from(item.to_owned()))
                .collect();

            let block = Block::bordered()
                .title_top(" Open ".bold())
                .title_bottom(" (j,k|↓,↑) ━ (ENTER) ━ (ESC) ".bold())
                .title_alignment(Alignment::Center)
                .style(
                    Style::new()
                        .fg(Color::Indexed(args.colors.main_text_color))
                        .bg(Color::Indexed(args.colors.popup_bg_color)),
                )
                .border_set(symbols::border::THICK)
                .border_style(Style::new().fg(Color::Indexed(args.colors.keyword_color)));

            let list = List::new(list_items).block(block).highlight_style(
                Style::new()
                    // .fg(Color::Indexed(args.colors.entry_color))
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED),
            );

            let popup_width = frame.area().width / 2;
            let popup_heigth = list.len() + 2;
            let popup_area = popup_area(frame.area(), popup_width, popup_heigth as u16);

            frame.render_widget(Clear, popup_area);
            frame.render_stateful_widget(list, popup_area, &mut app.bibiman.popup_area.popup_state)
        }
        None => {}
    }
}

pub fn render_header(args: &CLIArgs, frame: &mut Frame, rect: Rect) {
    let main_header = Paragraph::new("BIBIMAN – BibLaTeX manager TUI")
        .bold()
        .fg(Color::Indexed(args.colors.entry_color))
        .centered();
    frame.render_widget(main_header, rect)
}

pub fn render_footer(app: &mut App, args: &CLIArgs, frame: &mut Frame, rect: Rect) {
    let search_title = {
        match app.bibiman.former_area {
            Some(FormerArea::EntryArea) => "Search Entries: ".to_string(),
            Some(FormerArea::TagArea) => "Search Keywords: ".to_string(),
            _ => " Search ".to_string(),
        }
    };

    let title_lenght: u16 = search_title.chars().count() as u16;

    let block = Block::new()
        .padding(Padding::horizontal(1))
        .bg(Color::Indexed(args.colors.bar_bg_color));

    let search_string = Paragraph::new(Line::from(vec![
        Span::styled(
            search_title,
            if let Some(FormerArea::EntryArea) = app.bibiman.former_area {
                Style::new()
                    .fg(Color::Indexed(args.colors.entry_color))
                    .add_modifier(Modifier::BOLD)
            } else if let Some(FormerArea::TagArea) = app.bibiman.former_area {
                Style::new()
                    .fg(Color::Indexed(args.colors.keyword_color))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::new()
                    .fg(Color::Indexed(args.colors.highlight_text_color))
                    .add_modifier(Modifier::BOLD)
            },
        ),
        Span::raw(app.bibiman.search_struct.search_string.clone())
            .fg(Color::Indexed(args.colors.highlight_text_color)),
    ]))
    .block(block);

    render_cursor(app, frame, rect, title_lenght + 1);
    frame.render_widget(search_string, rect);
}

// Render info of the current file and process
// 1. Basename of the currently loaded file
// 2. Keyword by which the entries are filtered at the moment
// 3. Currently selected entry and total count of entries
pub fn render_file_info(app: &mut App, args: &CLIArgs, frame: &mut Frame, rect: Rect) {
    let block = Block::new() // can also be Block::new
        // Leave Top empty to simulate one large box with borders of entry list
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_set(if let CurrentArea::EntryArea = app.bibiman.current_area {
            symbols::border::THICK
        } else {
            symbols::border::PLAIN
        })
        .border_style(if let CurrentArea::EntryArea = app.bibiman.current_area {
            Style::new().fg(Color::Indexed(args.colors.highlight_text_color))
        } else {
            Style::new()
                .fg(Color::Indexed(args.colors.entry_color))
                .add_modifier(Modifier::BOLD)
        });

    frame.render_widget(block, rect);

    let [file_area, keyword_area, count_area] = Layout::horizontal([
        Constraint::Fill(3),
        Constraint::Fill(4),
        Constraint::Fill(1),
    ])
    .horizontal_margin(1)
    .areas(rect);

    let file_info = if args.pos_args.len() == 1 && args.pos_args.first().unwrap().is_file() {
        Line::from(vec![
            Span::raw("File: ")
                .fg(Color::Indexed(args.colors.main_text_color))
                .bold(),
            Span::raw(args.pos_args[0].file_name().unwrap().to_string_lossy())
                .fg(Color::Indexed(args.colors.main_text_color))
                .bold(),
        ])
        .bg(Color::Indexed(args.colors.bar_bg_color))
    } else if args.pos_args.len() == 1 && args.pos_args.first().unwrap().is_dir() {
        Line::from(vec![
            Span::raw("Directory: ")
                .bold()
                .fg(Color::Indexed(args.colors.main_text_color)),
            Span::raw(args.pos_args[0].file_name().unwrap().to_string_lossy())
                .fg(Color::Indexed(args.colors.main_text_color))
                .bold(),
            Span::raw("/*.bib")
                .fg(Color::Indexed(args.colors.main_text_color))
                .bold(),
        ])
        .bg(Color::Indexed(args.colors.bar_bg_color))
    } else {
        Line::from(vec![
            Span::raw("Multiple files (")
                .fg(Color::Indexed(args.colors.main_text_color))
                .bold(),
            Span::raw(count_files(&args.files).to_string())
                .fg(Color::Indexed(args.colors.main_text_color))
                .bold(),
            Span::raw(")")
                .fg(Color::Indexed(args.colors.main_text_color))
                .bold(),
        ])
        .bg(Color::Indexed(args.colors.bar_bg_color))
    };

    let cur_keywords = Line::from(if !app.bibiman.tag_list.selected_keywords.is_empty() {
        vec![
            Span::raw("Selected keywords: ").fg(Color::Indexed(args.colors.main_text_color)),
            // Show all keywords in correct order if list is filtered
            // successively by multiple keywords
            Span::raw(app.bibiman.tag_list.selected_keywords.join(" → "))
                .bold()
                .green(),
        ]
    } else {
        vec![Span::raw(" ")]
    })
    .bg(Color::Indexed(args.colors.bar_bg_color));
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
                .fg(Color::Indexed(args.colors.main_text_color))
                .bold(),
                Span::raw("/").fg(Color::Indexed(args.colors.main_text_color)),
                Span::raw(app.bibiman.entry_table.entry_table_items.len().to_string())
                    .fg(Color::Indexed(args.colors.main_text_color)),
            ]
        } else {
            vec![Span::raw("No entries")]
        },
    )
    .right_aligned()
    .bg(Color::Indexed(args.colors.bar_bg_color));
    frame.render_widget(file_info, file_area);
    frame.render_widget(cur_keywords, keyword_area);
    frame.render_widget(item_count, count_area);
}

pub fn render_entrytable(app: &mut App, args: &CLIArgs, frame: &mut Frame, rect: Rect) {
    let entry_box_selected_border_style: Style =
        Style::new().fg(Color::Indexed(args.colors.highlight_text_color));
    let entry_box_selected_title_style: Style = Style::new()
        .fg(Color::Indexed(args.colors.entry_color))
        .add_modifier(Modifier::BOLD);
    let entry_box_unselected_border_style: Style =
        Style::new().fg(Color::Indexed(args.colors.main_text_color));
    let entry_box_unselected_title_style: Style = Style::new()
        .fg(Color::Indexed(args.colors.entry_color))
        .add_modifier(Modifier::BOLD);
    let selected_table_col_style: Style = Style::new().add_modifier(Modifier::BOLD);
    let selectec_table_cell_style: Style = Style::new().add_modifier(Modifier::REVERSED);
    let entry_selected_row_style: Style = Style::new()
        .fg(Color::Indexed(args.colors.entry_color))
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::REVERSED);

    let block = Block::new() // can also be Block::new
        .title(
            Line::styled(
                " Bibliographic Entries ",
                if let CurrentArea::EntryArea = app.bibiman.current_area {
                    entry_box_selected_title_style
                } else {
                    entry_box_unselected_title_style
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
            entry_box_selected_border_style
        } else {
            entry_box_unselected_border_style
        });

    let header_style = Style::default()
        .bold()
        .fg(Color::Indexed(args.colors.main_text_color))
        .bg(Color::Indexed(args.colors.bar_bg_color));

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
                    Color::Indexed(args.colors.selected_row_bg_color)
                } else {
                    Color::Indexed(args.colors.bar_bg_color)
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
                    Color::Indexed(args.colors.selected_row_bg_color)
                } else {
                    Color::Indexed(args.colors.bar_bg_color)
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
                    Color::Indexed(args.colors.selected_row_bg_color)
                } else {
                    Color::Indexed(args.colors.bar_bg_color)
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
                    Color::Indexed(args.colors.selected_row_bg_color)
                } else {
                    Color::Indexed(args.colors.bar_bg_color)
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
        .map(|(i, data)| {
            let item = data.ref_vec();
            item.into_iter()
                .map(|content| Cell::from(Text::from(content.to_string())))
                .collect::<Row>()
                .style(
                    Style::new().fg(color_list(
                        args,
                        i as i32,
                        app.bibiman
                            .entry_table
                            .entry_table_state
                            .selected()
                            .unwrap_or(0) as i32,
                        args.colors.highlight_text_color,
                        20,
                    )),
                )
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
    .row_highlight_style(entry_selected_row_style)
    .column_highlight_style(selected_table_col_style)
    .cell_highlight_style(selectec_table_cell_style)
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

pub fn render_selected_item(app: &mut App, args: &CLIArgs, frame: &mut Frame, rect: Rect) {
    // We get the info depending on the item's state.
    let style_value = Style::new()
        .bold()
        .fg(Color::Indexed(args.colors.main_text_color));
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
                Span::styled(
                    cur_entry.authors(),
                    Style::new().fg(Color::Indexed(args.colors.info_color)),
                ),
            ]));
            if cur_entry.subtitle.is_some() {
                lines.push(Line::from(vec![
                    Span::styled("Title: ", style_value),
                    Span::styled(
                        cur_entry.title(),
                        Style::new()
                            .fg(Color::Indexed(args.colors.entry_color))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    Span::styled(
                        ": ",
                        Style::new()
                            .fg(Color::Indexed(args.colors.entry_color))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    Span::styled(
                        cur_entry.subtitle(),
                        Style::new()
                            .fg(Color::Indexed(args.colors.entry_color))
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Title: ", style_value),
                    Span::styled(
                        cur_entry.title(),
                        Style::new()
                            .fg(Color::Indexed(args.colors.entry_color))
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled("Year: ", style_value),
                Span::styled(
                    cur_entry.year(),
                    Style::new().fg(Color::Indexed(args.colors.keyword_color)),
                ),
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
                    content.push(Span::raw("▐").fg(Color::Indexed(args.colors.bar_bg_color)));
                    content.push(Span::styled(
                        k,
                        Style::default()
                            .bg(Color::Indexed(args.colors.bar_bg_color))
                            .fg(
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
                                    Color::Indexed(args.colors.main_text_color)
                                },
                            ),
                    ));
                    content.push(Span::raw("▌").fg(Color::Indexed(args.colors.bar_bg_color)));
                }
                lines.push(Line::from(content))
            }
            if cur_entry.doi_url.is_some() || cur_entry.filepath.is_some() {
                lines.push(Line::raw(""));
            }
            if cur_entry.doi_url.is_some() {
                lines.push(Line::from(vec![
                    Span::styled("DOI/URL: ", style_value),
                    Span::styled(
                        cur_entry.doi_url(),
                        Style::new()
                            .fg(Color::Indexed(args.colors.main_text_color))
                            .underlined(),
                    ),
                ]));
            }
            if cur_entry.filepath.is_some() {
                lines.push(Line::from(vec![
                    Span::styled("File: ", style_value),
                    Span::styled(
                        cur_entry.filepath().to_string_lossy(),
                        Style::new().fg(Color::Indexed(args.colors.main_text_color)),
                    ),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                cur_entry.abstract_text.clone(),
                Style::new().fg(Color::Indexed(args.colors.main_text_color)),
            )]));
            lines
        } else {
            let lines = vec![
                Line::from(" "),
                "No entry selected".bold().into_centered_line().red(),
            ];
            lines
        }
    };
    let info = Text::from(lines);

    // We show the list item's info under the list in this paragraph
    let block = Block::bordered()
        .title(Line::raw(" Entry Information ").centered().bold())
        .border_set(symbols::border::PLAIN)
        .border_style(Style::new().fg(Color::Indexed(args.colors.main_text_color)))
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

pub fn render_taglist(app: &mut App, args: &CLIArgs, frame: &mut Frame, rect: Rect) {
    let keyword_box_selected_border_style: Style =
        Style::new().fg(Color::Indexed(args.colors.highlight_text_color));
    let keyword_box_selected_title_style: Style = Style::new()
        .fg(Color::Indexed(args.colors.keyword_color))
        .add_modifier(Modifier::BOLD);
    let keyword_box_unselected_border_style: Style =
        Style::new().fg(Color::Indexed(args.colors.main_text_color));
    let keyword_box_unselected_title_style: Style = Style::new()
        .fg(Color::Indexed(args.colors.keyword_color))
        .add_modifier(Modifier::BOLD);
    let keyword_selected_row_style: Style = Style::new()
        .fg(Color::Indexed(args.colors.keyword_color))
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::REVERSED);

    let block = Block::bordered()
        .title(
            Line::styled(
                " Keywords ",
                if let CurrentArea::TagArea = app.bibiman.current_area {
                    keyword_box_selected_title_style
                } else {
                    keyword_box_unselected_title_style
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
            keyword_box_selected_border_style
        } else {
            keyword_box_unselected_border_style
        });

    // Iterate through all elements in the `items` and stylize them.
    let items: Vec<ListItem> = app
        .bibiman
        .tag_list
        .tag_list_items
        .iter()
        .enumerate()
        .map(|(i, keyword)| {
            ListItem::from(keyword.to_owned()).style(Style::new().fg(
                if app.bibiman.tag_list.tag_list_state.selected().is_some() {
                    color_list(
                        args,
                        i as i32,
                        app.bibiman.tag_list.tag_list_state.selected().unwrap() as i32,
                        args.colors.highlight_text_color,
                        20,
                    )
                } else {
                    Color::Indexed(args.colors.main_text_color)
                },
            )) //.bg(color)
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let list = List::new(items)
        .block(block)
        .highlight_style(keyword_selected_row_style);

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
fn render_cursor(app: &mut App, frame: &mut Frame, area: Rect, x_offset: u16) {
    let scroll = app.input.visual_scroll(area.width as usize);
    if app.input_mode {
        let (x, y) = (
            area.x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + x_offset,
            area.bottom().saturating_sub(1),
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

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
