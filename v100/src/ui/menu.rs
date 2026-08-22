//! Menus: character creation and list-based selection (inventory, shop,
//! quests). These are drawn with ratatui and return the user's choice.

use crate::core::action::Action;
use crate::data::classes::{Class, Race};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

/// The result of a menu interaction.
pub enum MenuResult {
    /// The user selected an item at the given index.
    Selected(usize),
    /// The user cancelled (Esc).
    Cancelled,
}

/// Draw a character-creation screen and return the chosen race and class.
///
/// `selection` is a `(race_idx, class_idx)` pair that is advanced by the
/// caller as the user navigates.
pub fn render_character_creation(f: &mut Frame, area: Rect, race_idx: usize, class_idx: usize) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Race list.
    let races = Race::ALL;
    let race_items: Vec<ListItem> = races
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let style = if i == race_idx {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let bonuses = r.bonuses();
            let text = format!(
                "{}  (+{}/{}/{}/{}/{}/{})",
                r.name(),
                bonuses.0,
                bonuses.1,
                bonuses.2,
                bonuses.3,
                bonuses.4,
                bonuses.5
            );
            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();
    f.render_widget(
        List::new(race_items).block(Block::default().borders(Borders::ALL).title(" Race ")),
        chunks[0],
    );

    // Class list.
    let classes = Class::ALL;
    let class_items: Vec<ListItem> = classes
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let style = if i == class_idx {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let text = format!("{}  (HP {})", c.name(), c.base_hp());
            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();
    f.render_widget(
        List::new(class_items).block(Block::default().borders(Borders::ALL).title(" Class ")),
        chunks[1],
    );

    // Help text.
    let help = Paragraph::new(Line::from(Span::styled(
        "Arrow keys: navigate   Enter: confirm   Esc: quit",
        Style::default().fg(Color::Gray),
    )));
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    f.render_widget(help, help_area);
}

/// Draw a generic list menu with a title and a set of items. Returns nothing;
/// the caller handles selection. `selected` highlights the current choice.
pub fn render_list_menu(f: &mut Frame, area: Rect, title: &str, items: &[String], selected: usize) {
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let style = if i == selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(text.clone(), style)))
        })
        .collect();
    f.render_widget(
        List::new(list_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {}  (Esc to cancel) ", title)),
        ),
        area,
    );
}

/// A full-screen overlay (death, victory, help).
pub fn render_overlay(f: &mut Frame, area: Rect, title: &str, body: &[String], color: Color) {
    let lines: Vec<Line> = body
        .iter()
        .map(|s| Line::from(Span::styled(s.clone(), Style::default().fg(color))))
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(color));
    f.render_widget(Paragraph::new(lines).block(block), area);
}

/// The action to take when a menu item is selected (for the inventory menu).
pub fn inventory_action_for(index: usize, verb: InventoryVerb) -> Action {
    match verb {
        InventoryVerb::Drop => Action::Drop { index },
        InventoryVerb::Quaff => Action::Quaff { index },
        InventoryVerb::Eat => Action::Eat { index },
        InventoryVerb::Read => Action::Read { index },
        InventoryVerb::Wield => Action::Wield { index },
        InventoryVerb::Wear => Action::Wear { index },
        InventoryVerb::RingOn => Action::RingOn { index },
        InventoryVerb::RingOff => Action::RingOff { index },
        InventoryVerb::Identify => Action::Identify { index },
    }
}

/// The verb for an inventory menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryVerb {
    Drop,
    Quaff,
    Eat,
    Read,
    Wield,
    Wear,
    RingOn,
    RingOff,
    Identify,
}

impl InventoryVerb {
    pub fn name(self) -> &'static str {
        match self {
            InventoryVerb::Drop => "Drop",
            InventoryVerb::Quaff => "Quaff",
            InventoryVerb::Eat => "Eat",
            InventoryVerb::Read => "Read",
            InventoryVerb::Wield => "Wield",
            InventoryVerb::Wear => "Wear",
            InventoryVerb::RingOn => "Ring On",
            InventoryVerb::RingOff => "Ring Off",
            InventoryVerb::Identify => "Identify",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_action_maps_correctly() {
        assert_eq!(
            inventory_action_for(3, InventoryVerb::Quaff),
            Action::Quaff { index: 3 }
        );
        assert_eq!(
            inventory_action_for(0, InventoryVerb::Wield),
            Action::Wield { index: 0 }
        );
    }

    #[test]
    fn verb_names_are_distinct() {
        let names: Vec<&str> = [
            InventoryVerb::Drop,
            InventoryVerb::Quaff,
            InventoryVerb::Eat,
            InventoryVerb::Read,
            InventoryVerb::Wield,
            InventoryVerb::Wear,
            InventoryVerb::RingOn,
            InventoryVerb::RingOff,
            InventoryVerb::Identify,
        ]
        .iter()
        .map(|v| v.name())
        .collect();
        assert_eq!(
            names.len(),
            names
                .into_iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
        );
    }
}
