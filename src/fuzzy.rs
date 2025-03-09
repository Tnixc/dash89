use std::collections::HashSet;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::widgets::ListState;

use crate::app::App;
use crate::config::{Widget, WidgetType};
use crate::ui::Window;

pub struct Matcher {
    matcher: SkimMatcherV2,
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn match_items<'a>(&self, query: &str, items: &'a [String]) -> Vec<(i64, &'a String)> {
        let mut matches: Vec<_> = items
            .iter()
            .filter_map(|item| {
                self.matcher
                    .fuzzy_match(item, query)
                    .map(|score| (score, item))
            })
            .collect();

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.0.cmp(&a.0));
        matches
    }
}

impl App {
    pub fn enter_fuzzy_search(&mut self) {
        self.mode = Window::FuzzySearch;
        // Initialize matches with all available topics
        self.fuzzy_search.update_matches(&self.available_topics);
    }

    pub fn exit_fuzzy_search(&mut self) {
        self.mode = Window::Main;
        self.fuzzy_search.input.clear();
    }

    pub fn handle_search_selection(&mut self) -> Option<String> {
        if let Some(selected_topic) = self.fuzzy_search.get_selected().cloned() {
            // Create a new widget in the first available grid position
            let widget = Widget {
                topic: selected_topic.clone(),
                label: selected_topic.clone(),
                widget_type: WidgetType::Text,
                position: self.find_next_grid_position(),
            };

            let _ = self.add_widget(widget);
            self.exit_fuzzy_search();
            Some(selected_topic)
        } else {
            None
        }
    }
}

pub struct FuzzySearch {
    pub input: String,
    pub matcher: Matcher,
    pub matches: Vec<String>,
    pub list_state: ListState,
}

impl FuzzySearch {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            input: String::new(),
            matcher: Matcher::new(),
            matches: Vec::new(),
            list_state,
        }
    }

    pub fn update_matches(&mut self, available_topics: &HashSet<String>) {
        let vec = available_topics.iter().cloned().collect::<Vec<_>>();
        if self.input.is_empty() {
            // If empty query, show all topics
            self.matches = vec;
        } else {
            // Otherwise do fuzzy search
            let matches = self.matcher.match_items(&self.input, &vec);
            self.matches = matches.into_iter().map(|(_, item)| item.clone()).collect();
        }

        // Reset selection or adjust if out of bounds
        if self.matches.is_empty() {
            self.list_state.select(None);
        } else {
            let current = self.list_state.selected().unwrap_or(0);
            if current >= self.matches.len() {
                self.list_state.select(Some(self.matches.len() - 1));
            }
        }
    }

    pub fn get_selected(&self) -> Option<&String> {
        self.list_state.selected().and_then(|i| self.matches.get(i))
    }

    pub fn move_selection(&mut self, offset: isize) {
        if self.matches.is_empty() {
            return;
        }

        let len = self.matches.len();
        let current = self.list_state.selected().unwrap_or(0);

        let new_index = if offset.is_negative() {
            if current == 0 {
                len - 1 // Wrap to end
            } else {
                current - offset.unsigned_abs() as usize
            }
        } else {
            (current + offset as usize) % len
        };

        self.list_state.select(Some(new_index));
    }
}
