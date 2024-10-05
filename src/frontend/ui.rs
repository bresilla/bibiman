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
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Cell, HighlightSpacing, List, ListItem, Padding, Paragraph, Row, StatefulWidget,
        Table, Widget, Wrap,
    },
};

use crate::{backend::bib::BibiEntry, frontend::app::App, frontend::keywords::TagListItem};

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

        let [tag_area, info_area] =
            Layout::horizontal([Constraint::Max(25), Constraint::Min(35)]).areas(item_area);

        // Render header and footer
        App::render_header(header_area, buf);
        self.render_footer(footer_area, buf);
        // Render list area where entry gets selected
        self.render_entrytable(list_area, buf);
        // Render infos related to selected entry
        // TODO: only placeholder at the moment, has to be impl.
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
                    .border_set(symbols::border::ROUNDED);
                Paragraph::new(Line::from(vec![
                    Span::styled("j/k: ", style_emph),
                    Span::raw("to move | "),
                    Span::styled("g/G: ", style_emph),
                    Span::raw("go top/bottom | "),
                    Span::styled("TAB: ", style_emph),
                    Span::raw("switch fields | "),
                    Span::styled("y: ", style_emph),
                    Span::raw("yank citekey | "),
                    Span::styled("e: ", style_emph),
                    Span::raw("edit entry | "),
                    Span::styled("/: ", style_emph),
                    Span::raw("search"),
                ]))
                .block(block)
                .centered()
                .render(area, buf);
            }
        }
    }

    pub fn render_entrytable(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered() // can also be Block::new
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
        // .bg(Color::Black); // .bg(NORMAL_ROW_BG);
        let header_style = Style::default().bold().fg(TEXT_FG_COLOR);
        // let selected_style = Style::default()
        //     .add_modifier(Modifier::REVERSED)
        //     .add_modifier(Modifier::BOLD);

        let header = [
            "Authors".underlined(),
            "Title".underlined(),
            "Year".underlined(),
            "Type".underlined(),
        ]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
        // Iterate over vector storing each entries data fields
        let rows = self
            .entry_table
            .entry_table_items
            .iter()
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
    }

    pub fn render_selected_item(&mut self, area: Rect, buf: &mut Buffer) {
        // We get the info depending on the item's state.
        // TODO: Implement logic showin informations for selected entry:
        let style_value = Style::new().bold().fg(TEXT_FG_COLOR);
        let lines = {
            // if self.entry_table.entry_table_items.len() > 0 {
            if self.entry_table.entry_table_state.selected().is_some() {
                let mut lines = vec![];
                lines.push(Line::from(vec![
                    Span::styled("Authors: ", style_value),
                    Span::styled(
                        String::from(BibiEntry::get_authors(
                            &self.get_selected_citekey(),
                            &self.main_biblio.bibliography,
                        )),
                        Style::new().green(),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Title: ", style_value),
                    Span::styled(
                        String::from(BibiEntry::get_title(
                            &self.get_selected_citekey(),
                            &self.main_biblio.bibliography,
                        )),
                        Style::new().magenta(),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Year: ", style_value),
                    Span::styled(
                        String::from(BibiEntry::get_year(
                            &self.get_selected_citekey(),
                            &self.main_biblio.bibliography,
                        )),
                        Style::new().light_magenta(),
                    ),
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    String::from(BibiEntry::get_abstract(
                        &self.get_selected_citekey(),
                        &self.main_biblio.bibliography,
                    )),
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
            if self.scroll_info == 0 {
                self.scroll_info
            } else if area.height > box_height as u16 {
                self.scroll_info = 0;
                self.scroll_info
            } else if self.scroll_info > (box_height as u16 + 1 - area.height) {
                self.scroll_info = box_height as u16 + 1 - area.height;
                self.scroll_info
            } else {
                self.scroll_info
            }
        };

        // We can now render the item info
        Paragraph::new(info)
            .block(block)
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
                ListItem::from(todo_item) //.bg(color)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            // .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.tag_list.tag_list_state);
    }
}
