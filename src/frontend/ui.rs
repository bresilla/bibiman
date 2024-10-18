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

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Cell, HighlightSpacing, List, ListItem, Padding, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, StatefulWidget, Table, Widget, Wrap,
    },
};

use crate::{
    backend::bib::BibiMain,
    frontend::{app::App, keywords::TagListItem},
};

use super::app::{CurrentArea, FormerArea};

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
const SELECTED_STYLE: Style = Style::new()
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::REVERSED);
const TEXT_FG_COLOR: Color = Color::Indexed(252);
const TEXT_UNSELECTED_FG_COLOR: Color = Color::Indexed(245);
const SORTED_ENTRIES: &str = "▼";
const SORTED_ENTRIES_REVERSED: &str = "▲";

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

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        let [entry_area, entry_info_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(list_area);

        let [tag_area, info_area] =
            Layout::horizontal([Constraint::Max(25), Constraint::Min(35)]).areas(item_area);

        // Render header and footer
        App::render_header(header_area, buf);
        self.render_footer(footer_area, buf);
        // Render list area where entry gets selected
        self.render_entrytable(entry_area, buf);
        self.render_file_info(entry_info_area, buf);
        // Render infos related to selected entry
        self.render_taglist(tag_area, buf);
        self.render_selected_item(info_area, buf);
    }
}

impl App {
    pub fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("BIBIMAN – BibLaTeX manager TUI")
            .bold()
            .fg(MAIN_BLUE_COLOR)
            .centered()
            .render(area, buf);
    }

    pub fn render_footer(&mut self, area: Rect, buf: &mut Buffer) {
        match &self.current_area {
            CurrentArea::SearchArea => {
                let search_title = {
                    match self.former_area {
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
                Paragraph::new(self.search_struct.search_string.clone())
                    .block(block)
                    .render(area, buf);
            }
            _ => {
                let style_emph = Style::new().bold().fg(TEXT_FG_COLOR);
                let block = Block::bordered()
                    .title(Line::raw(" Basic Commands ").centered())
                    .border_style(BOX_UNSELECTED_BORDER_STYLE)
                    .border_set(symbols::border::PLAIN);
                Paragraph::new(Line::from(vec![
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
                .centered()
                .render(area, buf);
            }
        }
    }

    // Render info of the current file and process
    // 1. Basename of the currently loaded file
    // 2. Keyword by which the entries are filtered at the moment
    // 3. Currently selected entry and total count of entries
    pub fn render_file_info(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new() // can also be Block::new
            // Leave Top empty to simulate one large box with borders of entry list
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_set(if let CurrentArea::EntryArea = self.current_area {
                symbols::border::THICK
            } else {
                symbols::border::PLAIN
            })
            .border_style(if let CurrentArea::EntryArea = self.current_area {
                BOX_SELECTED_BOX_STYLE
            } else {
                BOX_UNSELECTED_BORDER_STYLE
            });

        let background_style = Color::Indexed(235);

        let [file_area, keyword_area, count_area] = Layout::horizontal([
            Constraint::Fill(4),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ])
        .horizontal_margin(1)
        .areas(area);

        Line::from(vec![
            Span::raw("File: ").bold(),
            Span::raw(self.main_bibfile.file_name().unwrap().to_string_lossy()).bold(),
        ])
        .bg(background_style)
        .render(file_area, buf);

        Line::from(if !self.tag_list.selected_keyword.is_empty() {
            vec![
                Span::raw("Filtered by: "),
                Span::raw(self.tag_list.selected_keyword.to_string())
                    .bold()
                    .green(),
            ]
        } else {
            vec![Span::raw(" ")]
        })
        .bg(background_style)
        .render(keyword_area, buf);

        Line::from(if self.entry_table.entry_table_state.selected().is_some() {
            vec![
                Span::raw((self.entry_table.entry_table_state.selected().unwrap() + 1).to_string())
                    .bold(),
                Span::raw("/"),
                Span::raw(self.entry_table.entry_table_items.len().to_string()),
            ]
        } else {
            vec![Span::raw("No entries")]
        })
        .right_aligned()
        .bg(background_style)
        .render(count_area, buf);
        // Paragraph::new(Line::from(vec![Span::raw(
        //     self.main_bibfile.display().to_string(),
        // )]))
        // .block(block)
        // .render(area, buf);
        Widget::render(block, area, buf)
    }

    pub fn render_entrytable(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new() // can also be Block::new
            .title(
                Line::styled(
                    " Bibliographic Entries ",
                    if let CurrentArea::EntryArea = self.current_area {
                        BOX_SELECTED_TITLE_STYLE
                    } else {
                        BOX_UNSELECTED_TITLE_STYLE
                    },
                )
                .centered(),
            )
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
            .border_set(if let CurrentArea::EntryArea = self.current_area {
                symbols::border::THICK
            } else {
                symbols::border::PLAIN
            })
            .border_style(if let CurrentArea::EntryArea = self.current_area {
                BOX_SELECTED_BOX_STYLE
            } else {
                BOX_UNSELECTED_BORDER_STYLE
            });

        let header_style = Style::default().bold().fg(TEXT_FG_COLOR);

        let header = Row::new(vec![
            Cell::from(Line::from(vec![
                Span::raw("Author").underlined(),
                Span::raw(format!(
                    " {}",
                    if self.entry_table.entry_table_reversed_sort {
                        SORTED_ENTRIES_REVERSED
                    } else {
                        SORTED_ENTRIES
                    }
                )),
            ])),
            Cell::from("Title".to_string().underlined()),
            Cell::from("Year".to_string().underlined()),
            Cell::from("Type".to_string().underlined()),
        ])
        .style(header_style)
        .height(1);

        // Iterate over vector storing each entries data fields
        let rows = self
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
                Constraint::Length(4),
                Constraint::Percentage(10),
            ],
        )
        .block(block)
        .header(header)
        .column_spacing(2)
        .highlight_style(SELECTED_STYLE)
        // .bg(Color::Black)
        .highlight_spacing(HighlightSpacing::Always);
        StatefulWidget::render(
            entry_table,
            area,
            buf,
            &mut self.entry_table.entry_table_state,
        );

        // Scrollbar for entry table
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(None)
            .begin_symbol(SCROLLBAR_UPPER_CORNER)
            .end_symbol(None)
            .thumb_style(Style::new().fg(Color::DarkGray));

        if let CurrentArea::EntryArea = self.current_area {
            // render the scrollbar
            StatefulWidget::render(
                scrollbar,
                area,
                buf,
                &mut self.entry_table.entry_scroll_state,
            );
        }
    }

    pub fn render_selected_item(&mut self, area: Rect, buf: &mut Buffer) {
        // We get the info depending on the item's state.
        let style_value = Style::new().bold().fg(TEXT_FG_COLOR);
        let style_value_sec = Style::new()
            .add_modifier(Modifier::ITALIC)
            .fg(TEXT_FG_COLOR);
        let lines = {
            // if self.entry_table.entry_table_items.len() > 0 {
            if self.entry_table.entry_table_state.selected().is_some() {
                let idx = self.entry_table.entry_table_state.selected().unwrap();
                let cur_entry = &self.entry_table.entry_table_items[idx];
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
            // .borders(Borders::TOP)
            .border_set(symbols::border::PLAIN)
            .border_style(BOX_UNSELECTED_BORDER_STYLE)
            // .bg(Color::Black)
            .padding(Padding::horizontal(1));

        // INFO: '.line_count' method only possible with unstable-rendered-line-info feature -> API might change: https://github.com/ratatui/ratatui/issues/293#ref-pullrequest-2027056434
        let box_height = Paragraph::new(info.clone())
            .block(block.clone())
            .wrap(Wrap { trim: false })
            .line_count(area.width);
        // Make sure to allow scroll only if text is larger than the rendered area and stop scrolling when last line is reached
        let scroll_height = {
            if self.entry_table.entry_info_scroll == 0 {
                self.entry_table.entry_info_scroll
            } else if area.height > box_height as u16 {
                self.entry_table.entry_info_scroll = 0;
                self.entry_table.entry_info_scroll
            } else if self.entry_table.entry_info_scroll > (box_height as u16 + 2 - area.height) {
                self.entry_table.entry_info_scroll = box_height as u16 + 2 - area.height;
                self.entry_table.entry_info_scroll
            } else {
                self.entry_table.entry_info_scroll
            }
        };

        // We can now render the item info
        Paragraph::new(info)
            .block(
                block
                    // Render arrows to show that info box has content outside the block
                    .title(
                        Title::from(
                            if box_height > area.height.into()
                                && self.entry_table.entry_info_scroll
                                    < box_height as u16 + 2 - area.height
                            {
                                " ▼ "
                            } else {
                                ""
                            },
                        )
                        .position(Position::Bottom)
                        .alignment(Alignment::Right),
                    )
                    .title(
                        Title::from(if scroll_height > 0 { " ▲ " } else { "" })
                            .position(Position::Top)
                            .alignment(Alignment::Right),
                    ),
            )
            // .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .scroll((scroll_height, 0))
            .render(area, buf);
    }

    pub fn render_taglist(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(
                Line::styled(
                    " Keywords ",
                    if let CurrentArea::TagArea = self.current_area {
                        BOX_SELECTED_TITLE_STYLE
                    } else {
                        BOX_UNSELECTED_TITLE_STYLE
                    },
                )
                .centered(),
            )
            .border_set(if let CurrentArea::TagArea = self.current_area {
                symbols::border::THICK
            } else {
                symbols::border::PLAIN
            })
            .border_style(if let CurrentArea::TagArea = self.current_area {
                BOX_SELECTED_BOX_STYLE
            } else {
                BOX_UNSELECTED_BORDER_STYLE
            });
        // .bg(Color::Black);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
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
            .highlight_style(SELECTED_STYLE)
            // .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        // Save list length for calculating scrollbar need
        // Add 2 to compmensate lines of the block border
        let list_length = list.len() + 2;

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.tag_list.tag_list_state);

        // Scrollbar for keyword list
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(None)
            .begin_symbol(SCROLLBAR_UPPER_CORNER)
            .end_symbol(SCROLLBAR_LOWER_CORNER)
            .thumb_style(Style::new().fg(Color::DarkGray));

        if list_length > area.height.into() {
            if let CurrentArea::TagArea = self.current_area {
                // render the scrollbar
                StatefulWidget::render(scrollbar, area, buf, &mut self.tag_list.tag_scroll_state);
            }
        }
    }
}
