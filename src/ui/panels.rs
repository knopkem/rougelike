//! Overlay panels: inventory, character, help, history.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::core::game::Game;
use crate::ui::picker::Picker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Panel {
    Inventory,
    Character,
    Help,
    History,
    #[default]
    None,
}

fn center_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    }
}

pub fn render(frame: &mut Frame, game: &Game, panel: &Panel) {
    let area = frame.area();
    match panel {
        Panel::Inventory => render_inventory(frame, game, area),
        Panel::Character => render_character(frame, game, area),
        Panel::Help => render_help(frame, area),
        Panel::History => render_history(frame, game, area),
        Panel::None => {}
    }
}

fn render_inventory(frame: &mut Frame, game: &Game, area: Rect) {
    let p = &game.player;
    let mut rows: Vec<String> = Vec::new();
    if let Some(w) = &p.wielded {
        rows.push(format!("  wielded: {}", w.name()));
    }
    if let Some(a) = &p.armor {
        rows.push(format!("  armor:   {}", a.name()));
    }
    for (i, item) in p.inventory.iter().enumerate() {
        rows.push(format!("  ({}) {}", i, item.name()));
    }
    if p.inventory.is_empty() {
        rows.push(String::from("  (empty)"));
    }
    let h = rows.len() as u16 + 2;
    let rect = center_rect(40, h.min(area.height.saturating_sub(2)).max(4), area);
    let items: Vec<ListItem> = rows
        .iter()
        .map(|s| ListItem::new(s.as_str()).style(Style::default().fg(Color::White)))
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title("Inventory"),
    );
    frame.render_widget(list, rect);
}

pub fn render_picker(frame: &mut Frame, picker: &Picker, area: Rect) {
    let rows: Vec<String> = picker
        .rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let prefix = if i == picker.cursor { ">>" } else { "  " };
            format!("{}{}", prefix, r.label)
        })
        .collect();
    let items: Vec<ListItem> = rows
        .iter()
        .map(|s| ListItem::new(s.as_str()).style(Style::default().fg(Color::White)))
        .collect();
    let h = rows.len() as u16 + 2;
    let rect = center_rect(44, h.min(area.height.saturating_sub(2)).max(4), area);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(picker.kind.prompt()),
        );
    frame.render_widget(list, rect);
}

fn render_character(frame: &mut Frame, game: &Game, area: Rect) {
    let p = &game.player;
    let lines = vec![
        Line::from(Span::raw(format!("{} the {}", p.name, p.race))),
        Line::from(Span::raw(format!("Class: {}  Level: {}", p.class, p.level))),
        Line::from(Span::raw(format!(
            "HP: {}/{}   EP: {}/{}",
            p.hp, p.max_hp, p.ep, p.max_ep
        ))),
        Line::from(Span::raw(format!(
            "STR {}  DEX {}  CON {}  INT {}  WIS {}  CHA {}",
            p.str, p.dex, p.con, p.int, p.wis, p.cha
        ))),
        Line::from(Span::raw(format!("AC {}  to-hit {}", p.ac(), p.to_hit()))),
        Line::from(Span::raw(format!(
            "XP: {} / {}   Gold: {}",
            p.xp,
            p.xp_next(),
            p.gold
        ))),
    ];
    let h = lines.len() as u16 + 2;
    let rect = center_rect(46, h, area);
    let para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Character"),
    );
    frame.render_widget(para, rect);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::raw("  hjkl/yubn or arrows: move / attack")),
        Line::from(Span::raw("  . : wait          > : descend   < : ascend")),
        Line::from(Span::raw("  g : pick up item  (gold is picked up on walk)")),
        Line::from(Span::raw("  i : inventory   c : character  H : history")),
        Line::from(Span::raw("  ? : help          M : mute      q : quit (autosaves)")),
        Line::from(Span::raw("  Items (arrow/jk to pick, Enter, Esc cancels):")),
        Line::from(Span::raw("    U : use            V : quaff       E : eat")),
        Line::from(Span::raw("    D : drop           W : wield       Y : wear")),
        Line::from(Span::raw("    T : take off       P : ring on     O : ring off")),
        Line::from(Span::raw("    R : read           I : identify")),
        Line::from(Span::raw("    Z : fire wand      (arrows aim, Enter fires)")),
    ];
    let h = lines.len() as u16 + 2;
    let rect = center_rect(54, h, area);
    let para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title("Help"),
    );
    frame.render_widget(para, rect);
}

fn render_history(frame: &mut Frame, game: &Game, area: Rect) {
    let msgs = game.messages.tail(12);
    let lines: Vec<Line> = msgs
        .iter()
        .map(|m| {
            let color = match m.kind {
                crate::core::message::MessageKind::Combat => Color::Red,
                crate::core::message::MessageKind::Good => Color::Green,
                crate::core::message::MessageKind::Bad => Color::LightRed,
                crate::core::message::MessageKind::Quest => Color::Magenta,
                crate::core::message::MessageKind::System => Color::Cyan,
                crate::core::message::MessageKind::Normal => Color::White,
            };
            Line::from(
                Span::raw(format!("[{}] {}", m.turn, m.text)).style(Style::default().fg(color)),
            )
        })
        .collect();
    let rect = center_rect(60, 14, area);
    let para = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title("Message History"),
    );
    frame.render_widget(para, rect);
}
