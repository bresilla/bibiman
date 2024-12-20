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

#[derive(Debug, Clone)]
pub enum AppColorScheme {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub struct AppColors {
    pub main_text_color: u8,
    pub highlight_text_color: u8,
    pub entry_color: u8,
    pub keyword_color: u8,
    pub info_color: u8,
    pub confirm_color: u8,
    pub warn_color: u8,
    pub bar_bg_color: u8,
    pub popup_bg_color: u8,
    pub selected_row_bg_color: u8,
    pub color_scheme: AppColorScheme,
}

impl Default for AppColors {
    fn default() -> Self {
        Self {
            main_text_color: 250,
            highlight_text_color: 254,
            entry_color: 36,
            keyword_color: 101,
            info_color: 99,
            confirm_color: 47,
            warn_color: 124,
            bar_bg_color: 235,
            popup_bg_color: 234,
            selected_row_bg_color: 237,
            color_scheme: AppColorScheme::Dark,
        }
    }
}

impl AppColors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn main_text_color(&mut self, index: u8) {
        self.main_text_color = index
    }

    pub fn highlight_text_color(&mut self, index: u8) {
        self.highlight_text_color = index
    }

    pub fn entry_color(&mut self, index: u8) {
        self.entry_color = index
    }

    pub fn keyword_color(&mut self, index: u8) {
        self.keyword_color = index
    }

    pub fn info_color(&mut self, index: u8) {
        self.info_color = index
    }

    pub fn confirm_color(&mut self, index: u8) {
        self.confirm_color = index
    }

    pub fn warn_color(&mut self, index: u8) {
        self.warn_color = index
    }

    pub fn bar_bg_color(&mut self, index: u8) {
        self.bar_bg_color = index
    }

    pub fn popup_bg_color(&mut self, index: u8) {
        self.popup_bg_color = index
    }

    pub fn selected_row_bg_color(&mut self, index: u8) {
        self.selected_row_bg_color = index
    }

    pub fn toggle_color_scheme(&mut self) {
        match self.color_scheme {
            AppColorScheme::Light => self.color_scheme = AppColorScheme::Dark,
            AppColorScheme::Dark => self.color_scheme = AppColorScheme::Light,
        }
    }

    /// Activates the default color scheme for light background terminals
    pub fn light_colors(&mut self) {
        self.main_text_color(235);
        self.highlight_text_color(232);
        self.entry_color(23);
        self.keyword_color(58);
        self.info_color(57);
        self.bar_bg_color(144);
        self.popup_bg_color(187);
        self.confirm_color(22);
        self.selected_row_bg_color(107)
    }
}
