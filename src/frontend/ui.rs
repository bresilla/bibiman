use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{GRAY, SLATE},
        Color, Modifier, Style, Stylize,
    },
    symbols,
    text::{Line, Text},
    widgets::{
        Block, Cell, HighlightSpacing, List, ListItem, Padding, Paragraph, Row, StatefulWidget,
        Table, TableState, Widget, Wrap,
    },
};

use crate::frontend::app::{App, TagListItem};

use super::app::EntryTableItem;

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

impl From<&EntryTableItem> for ListItem<'_> {
    fn from(value: &EntryTableItem) -> Self {
        let line = Line::styled(format!("{}, {}", value.authors, value.title), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        let [tag_area, info_area] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]).areas(item_area);

        // Render header and footer
        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        // Render list area where entry gets selected
        // self.render_entry_table(list_area, buf);
        self.render_table(list_area, buf);
        // Render infos related to selected entry
        // TODO: only placeholder at the moment, has to be impl.
        self.render_taglist(tag_area, buf);
        self.render_selected_item(info_area, buf);
    }
}

impl App {
    pub fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Ratatui List Example")
            .bold()
            .centered()
            .render(area, buf);
    }

    pub fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use g/h to move, h to unselect, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    pub fn render_table(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(
                Line::raw(" Selection List ")
                    .centered()
                    .fg(Color::Indexed(39)),
            )
            // .borders(Borders::TOP)
            .border_set(symbols::border::ROUNDED)
            .border_style(BOX_BORDER_STYLE_MAIN)
            .bg(Color::Black); // .bg(NORMAL_ROW_BG);
        let header_style = Style::default().bold();
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

        let header = ["Authors".underlined(), "Title".underlined()]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
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
        let entry_table = Table::new(rows, [Constraint::Percentage(20), Constraint::Fill(1)])
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

    pub fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // We get the info depending on the item's state.
        // INFO: Only a placeholder at the moment:
        let info = "Infor for selected item".to_string();
        // TODO: Implement logic showin informations for selected entry:
        // let info = if let Some(i) = self.tag_list.state.selected() {
        //     "Infor for selected item".to_string()
        //     // match self.todo_list.items[i].status {
        //     //     Status::Completed => format!("✓ DONE: {}", self.todo_list.items[i].info),
        //     //     Status::Todo => format!("☐ TODO: {}", self.todo_list.items[i].info),
        //     // }
        // } else {
        //     "Nothing selected...".to_string()
        // };

        // We show the list item's info under the list in this paragraph
        let block = Block::bordered()
            .title(Line::raw(" Item Info ").centered())
            // .borders(Borders::TOP)
            .border_set(symbols::border::ROUNDED)
            .border_style(BOX_BORDER_STYLE_MAIN)
            .bg(Color::Black)
            .padding(Padding::horizontal(1));

        // We can now render the item info
        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    pub fn render_taglist(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::raw(" Tag List ").centered())
            .border_set(symbols::border::ROUNDED)
            .border_style(BOX_BORDER_STYLE_MAIN)
            .bg(Color::Black)
            .padding(Padding::horizontal(1));

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
