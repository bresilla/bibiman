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
    style::{
        palette::tailwind::{GRAY, SLATE},
        Color, Modifier, Style, Stylize,
    },
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, HighlightSpacing, List, ListItem, Padding, Paragraph, Row,
        StatefulWidget, Table, Widget, Wrap,
    },
};

use crate::{
    backend::bib::BibiEntry,
    frontend::app::{App, TagListItem},
};

use super::app::CurrentArea;

const MAIN_BLUE_COLOR: Color = Color::Indexed(39);
const MAIN_PURPLE_COLOR: Color = Color::Indexed(129);
const BOX_BORDER_STYLE_MAIN: Style = Style::new().fg(Color::White).bg(Color::Black);
const NORMAL_ROW_BG: Color = Color::Black;
const ALT_ROW_BG_COLOR: Color = Color::Indexed(234);
const SELECTED_STYLE: Style = Style::new()
    // .fg(MAIN_BLUE_COLOR)
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::REVERSED);
const TEXT_FG_COLOR: Color = SLATE.c200;

pub const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl From<&TagListItem> for ListItem<'_> {
    fn from(value: &TagListItem) -> Self {
        let line = Line::styled(format!("{}", value.info), TEXT_FG_COLOR);
        // match value.status {
        // Status::Todo => Line::styled(format!(" ☐ {}", value.todo), TEXT_FG_COLOR),
        // Status::Completed => {
        //     Line::styled(format!(" ✓ {}", value.todo), COMPLETED_TEXT_FG_COLOR)
        // }
        // };
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
            Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]).areas(item_area);

        // Render header and footer
        App::render_header(header_area, buf);
        self.render_footer(footer_area, buf);
        // Render list area where entry gets selected
        // self.render_entry_table(list_area, buf);
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
                let block = Block::bordered()
                    .title(" Search Entries ")
                    .border_set(symbols::border::ROUNDED);
                Paragraph::new(self.search_string.clone())
                    .block(block)
                    .render(area, buf);
            }
            _ => {
                let block = Block::bordered()
                    .title(Line::raw(" Basic Commands ").centered())
                    .border_set(symbols::border::ROUNDED);
                Paragraph::new(
                    "Use j/k to move, g/G to go top/bottom, y to yank the current citekey",
                )
                .block(block)
                .centered()
                .render(area, buf);
            }
        }
    }

    pub fn render_entrytable(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered() // can also be Block::new
            .title(
                Line::raw(" Selection List ")
                    .centered()
                    .fg(Color::Indexed(39)),
            )
            // .borders(Borders::TOP) // set borders for Block::new
            .border_set(symbols::border::ROUNDED)
            .border_style(BOX_BORDER_STYLE_MAIN)
            .bg(Color::Black); // .bg(NORMAL_ROW_BG);
        let header_style = Style::default().bold();
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

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
            .map(|(i, data)| {
                let item = data.ref_vec();
                item.into_iter()
                    .map(|content| Cell::from(Text::from(format!("{content}"))))
                    .collect::<Row>()
                    .style(Style::new().fg(Color::White).bg(alternate_colors(i)))
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
        .highlight_style(selected_style)
        .bg(Color::Black)
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
        let style_value = Style::new().bold();
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
            Style::default(),
        )]));
        let info = Text::from(lines);

        // We show the list item's info under the list in this paragraph
        let block = Block::bordered()
            .title(Line::raw(" Entry Information ").centered())
            // .borders(Borders::TOP)
            .border_set(symbols::border::ROUNDED)
            .border_style(BOX_BORDER_STYLE_MAIN)
            .bg(Color::Black)
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
            .title(Line::raw(" Tag List ").centered())
            .border_set(symbols::border::ROUNDED)
            .border_style(BOX_BORDER_STYLE_MAIN)
            .bg(Color::Black);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .tag_list
            .tag_list_items
            .iter()
            .enumerate()
            .map(|(i, todo_item)| {
                let color = alternate_colors(i);
                ListItem::from(todo_item).bg(color)
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
