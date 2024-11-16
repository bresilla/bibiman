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

use super::popup::PopupArea;
use crate::bibiman::entries::EntryTableColumn;
use crate::bibiman::{CurrentArea, FormerArea};
use crate::tui::popup::PopupKind;
use crate::App;
use crate::{
    MAIN_BLUE_COLOR_INDEX, MAIN_GREEN_COLOR_INDEX, MAIN_PURPLE_COLOR_INDEX, TEXT_FG_COLOR_INDEX,
    TEXT_HIGHLIGHT_COLOR_INDEX,
};
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

// Text colors
const TEXT_FG_COLOR: Color = Color::Indexed(TEXT_FG_COLOR_INDEX);
const TEXT_BRIGHT_FG_COLOR: Color = Color::Indexed(TEXT_HIGHLIGHT_COLOR_INDEX);
const MAIN_BLUE: Color = Color::Indexed(MAIN_BLUE_COLOR_INDEX);
const MAIN_PURPLE: Color = Color::Indexed(MAIN_PURPLE_COLOR_INDEX);
const MAIN_GREEN: Color = Color::Indexed(MAIN_GREEN_COLOR_INDEX);

// Background colors
const HEADER_FOOTER_BG: Color = Color::Indexed(235);
const POPUP_BG: Color = Color::Indexed(234);

// Box styles
// Keyword Box
const KEYWORD_BOX_SELECTED_BORDER_STYLE: Style = Style::new().fg(TEXT_BRIGHT_FG_COLOR);
const KEYWORD_BOX_SELECTED_TITLE_STYLE: Style =
    Style::new().fg(MAIN_PURPLE).add_modifier(Modifier::BOLD);
const KEYWORD_BOX_UNSELECTED_BORDER_STYLE: Style = Style::new().fg(TEXT_FG_COLOR);
const KEYWORD_BOX_UNSELECTED_TITLE_STYLE: Style =
    Style::new().fg(MAIN_PURPLE).add_modifier(Modifier::BOLD);
// Entry box
const ENTRY_BOX_SELECTED_BORDER_STYLE: Style = Style::new().fg(TEXT_BRIGHT_FG_COLOR);
const ENTRY_BOX_SELECTED_TITLE_STYLE: Style =
    Style::new().fg(MAIN_BLUE).add_modifier(Modifier::BOLD);
const ENTRY_BOX_UNSELECTED_BORDER_STYLE: Style = Style::new().fg(TEXT_FG_COLOR);
const ENTRY_BOX_UNSELECTED_TITLE_STYLE: Style =
    Style::new().fg(MAIN_BLUE).add_modifier(Modifier::BOLD);
// Default box
// const BOX_SELECTED_BORDER_STYLE: Style = Style::new().fg(TEXT_BRIGHT_FG_COLOR);
const BOX_SELECTED_TITLE_STYLE: Style = Style::new()
    .fg(TEXT_BRIGHT_FG_COLOR)
    .add_modifier(Modifier::BOLD);
const BOX_UNSELECTED_BORDER_STYLE: Style = Style::new().fg(TEXT_FG_COLOR);
// const BOX_UNSELECTED_TITLE_STYLE: Style =
// Style::new().fg(TEXT_FG_COLOR).add_modifier(Modifier::BOLD);
// Popup box
const POPUP_HELP_BOX: Style = Style::new().fg(TEXT_FG_COLOR).bg(POPUP_BG);

// Entry table styles
const ENTRY_SELECTED_ROW_STYLE: Style = Style::new()
    .fg(MAIN_BLUE)
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::REVERSED);
const KEYWORD_SELECTED_ROW_STYLE: Style = Style::new()
    .fg(MAIN_PURPLE)
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::REVERSED);
const SELECTION_SELECTED_ROW_STYLE: Style = Style::new()
    // .fg(MAIN_BLUE)
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::REVERSED);
const SELECTED_TABLE_COL_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);
const SELECTEC_TABLE_CELL_STYLE: Style = Style::new().add_modifier(Modifier::REVERSED);

// Symbols
const SORTED_ENTRIES: &str = "▼";
const SORTED_ENTRIES_REVERSED: &str = "▲";
const SCROLLBAR_UPPER_CORNER: Option<&str> = Some("┓");
const SCROLLBAR_LOWER_CORNER: Option<&str> = Some("┛");

// Info area styles
const INFO_STYLE_AUTHOR: Style = Style::new().fg(MAIN_GREEN);
const INFO_STYLE_TITLE: Style = Style::new().fg(MAIN_BLUE).add_modifier(Modifier::ITALIC);
const INFO_STYLE_YEAR: Style = Style::new().fg(MAIN_PURPLE);
const INFO_STYLE_DOI: Style = Style::new().fg(TEXT_FG_COLOR);
const INFO_STYLE_FILE: Style = Style::new().fg(TEXT_FG_COLOR);
const INFO_STYLE_ABSTRACT: Style = Style::new().fg(TEXT_FG_COLOR);

pub const fn color_list(list_item: i32, sel_item: i32, highlight: u8, max_diff: i32) -> Color {
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

pub fn render_ui(app: &mut App, frame: &mut Frame) {
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

    render_header(frame, header_area);
    if let CurrentArea::SearchArea = app.bibiman.current_area {
        render_footer(app, frame, footer_area);
    }
    render_entrytable(app, frame, entry_area);
    render_selected_item(app, frame, info_area);
    render_taglist(app, frame, tag_area);
    render_file_info(app, frame, entry_info_area);
    if app.bibiman.popup_area.is_popup {
        render_popup(app, frame);
    }
}

pub fn render_popup(app: &mut App, frame: &mut Frame) {
    match app.bibiman.popup_area.popup_kind {
        Some(PopupKind::Help) => {
            let block = Block::bordered()
                .title_top(" Keybindings ".bold())
                .title_bottom(" (j,k|↓,↑) ".bold())
                .title_alignment(Alignment::Center)
                .style(POPUP_HELP_BOX)
                .border_set(symbols::border::THICK)
                .border_style(Style::new().fg(MAIN_BLUE));

            let text: Text = PopupArea::popup_help();

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
        Some(PopupKind::Message) => {
            let area = frame.area();

            let block = Block::bordered()
                .title_top(" Message ".bold().fg(MAIN_GREEN))
                .border_style(Style::new().fg(MAIN_GREEN))
                .style(POPUP_HELP_BOX);

            let content = Paragraph::new(app.bibiman.popup_area.popup_message.clone())
                .block(block)
                .style(Style::new().fg(MAIN_GREEN));

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
                .title_bottom(" (j,k|↓,↑) ".bold())
                .title_alignment(Alignment::Center)
                .style(POPUP_HELP_BOX)
                .border_set(symbols::border::THICK)
                .border_style(Style::new().fg(MAIN_PURPLE));

            let list = List::new(list_items)
                .block(block)
                .highlight_style(SELECTION_SELECTED_ROW_STYLE);

            app.bibiman.popup_area.popup_state.select(Some(0));

            let popup_width = frame.area().width / 2;
            let popup_heigth = list.len() + 2;
            let popup_area = popup_area(frame.area(), popup_width, popup_heigth as u16);

            frame.render_widget(Clear, popup_area);
            frame.render_stateful_widget(list, popup_area, &mut app.bibiman.popup_area.popup_state)
            // let sized_list = SizedWrapper {
            //     inner: list.clone(),
            //     width: (frame.area().width / 2) as usize,
            //     height: list.len(),
            // };

            // let popup = Popup::new(sized_list)
            //     .title(" Select ".bold().into_centered_line().fg(MAIN_PURPLE))
            //     .border_set(symbols::border::THICK)
            //     // .border_style(Style::new().fg(MAIN_GREEN))
            //     .style(POPUP_HELP_BOX);

            // frame.render_stateful_widget(
            //     &popup,
            //     frame.area(),
            //     &mut app.bibiman.popup_area.popup_state,
            // )
        }
        None => {}
    }
}

pub fn render_header(frame: &mut Frame, rect: Rect) {
    let main_header = Paragraph::new("BIBIMAN – BibLaTeX manager TUI")
        .bold()
        .fg(MAIN_BLUE)
        .centered();
    frame.render_widget(main_header, rect)
}

pub fn render_footer(app: &mut App, frame: &mut Frame, rect: Rect) {
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
        .bg(HEADER_FOOTER_BG);

    let search_string = Paragraph::new(Line::from(vec![
        Span::styled(
            search_title,
            if let Some(FormerArea::EntryArea) = app.bibiman.former_area {
                ENTRY_BOX_SELECTED_TITLE_STYLE
            } else if let Some(FormerArea::TagArea) = app.bibiman.former_area {
                KEYWORD_BOX_SELECTED_TITLE_STYLE
            } else {
                BOX_SELECTED_TITLE_STYLE
            },
        ),
        Span::raw(app.bibiman.search_struct.search_string.clone()),
    ]))
    .block(block);

    render_cursor(app, frame, rect, title_lenght + 1);
    frame.render_widget(search_string, rect);
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
            ENTRY_BOX_SELECTED_BORDER_STYLE
        } else {
            ENTRY_BOX_UNSELECTED_BORDER_STYLE
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
                    ENTRY_BOX_SELECTED_TITLE_STYLE
                } else {
                    ENTRY_BOX_UNSELECTED_TITLE_STYLE
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
            ENTRY_BOX_SELECTED_BORDER_STYLE
        } else {
            ENTRY_BOX_UNSELECTED_BORDER_STYLE
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
        .map(|(i, data)| {
            let item = data.ref_vec();
            item.into_iter()
                .map(|content| Cell::from(Text::from(content.to_string())))
                .collect::<Row>()
                .style(
                    Style::new().fg(color_list(
                        i as i32,
                        app.bibiman
                            .entry_table
                            .entry_table_state
                            .selected()
                            .unwrap_or(0) as i32,
                        TEXT_HIGHLIGHT_COLOR_INDEX,
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
    .row_highlight_style(ENTRY_SELECTED_ROW_STYLE)
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
                Span::styled(cur_entry.authors(), INFO_STYLE_AUTHOR),
            ]));
            if cur_entry.subtitle.is_some() {
                lines.push(Line::from(vec![
                    Span::styled("Title: ", style_value),
                    Span::styled(cur_entry.title(), INFO_STYLE_TITLE),
                    Span::styled(": ", INFO_STYLE_TITLE),
                    Span::styled(cur_entry.subtitle(), INFO_STYLE_TITLE),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Title: ", style_value),
                    Span::styled(cur_entry.title(), INFO_STYLE_TITLE),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled("Year: ", style_value),
                Span::styled(cur_entry.year(), INFO_STYLE_YEAR),
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
            if cur_entry.doi_url.is_some() || cur_entry.filepath.is_some() {
                lines.push(Line::raw(""));
            }
            if cur_entry.doi_url.is_some() {
                lines.push(Line::from(vec![
                    Span::styled("DOI/URL: ", style_value),
                    Span::styled(cur_entry.doi_url(), INFO_STYLE_DOI.underlined()),
                ]));
            }
            if cur_entry.filepath.is_some() {
                lines.push(Line::from(vec![
                    Span::styled("File: ", style_value),
                    Span::styled(cur_entry.filepath(), INFO_STYLE_FILE),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                cur_entry.abstract_text.clone(),
                INFO_STYLE_ABSTRACT,
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
                    KEYWORD_BOX_SELECTED_TITLE_STYLE
                } else {
                    KEYWORD_BOX_UNSELECTED_TITLE_STYLE
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
            KEYWORD_BOX_SELECTED_BORDER_STYLE
        } else {
            KEYWORD_BOX_UNSELECTED_BORDER_STYLE
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
                        i as i32,
                        app.bibiman.tag_list.tag_list_state.selected().unwrap() as i32,
                        TEXT_HIGHLIGHT_COLOR_INDEX,
                        20,
                    )
                } else {
                    TEXT_FG_COLOR
                },
            )) //.bg(color)
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let list = List::new(items)
        .block(block)
        .highlight_style(KEYWORD_SELECTED_ROW_STYLE);

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
    // let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    // let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let vertical = Layout::vertical([Constraint::Length(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
