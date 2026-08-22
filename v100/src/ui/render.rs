//! Rendering: draws the `Game` state to the terminal using ratatui.
//!
//! Layout (top to bottom):
//! - A 1-line HUD bar (depth, theme, HP/EP, hunger, turn, gold).
//! - The main area: the level map (left) and a status/inventory panel (right).
//! - A message log (bottom).

use crate::core::color::Color as CoreColor;
use crate::core::events::Pos;
use crate::core::game::Game;
use crate::core::message::Severity;
use crate::data::themes::Theme;
use crate::map::level::{HEIGHT, Tile, WIDTH};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::palette::to_ratatui;

/// The main render entry point.
pub fn render(f: &mut Frame, game: &Game) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // HUD
            Constraint::Min(0),    // main
            Constraint::Length(4), // messages
        ])
        .split(area);

    render_hud(f, game, chunks[0]);
    render_main(f, game, chunks[1]);
    render_messages(f, game, chunks[2]);
}

/// The top HUD bar.
fn render_hud(f: &mut Frame, game: &Game, area: Rect) {
    let theme = Theme::for_depth(game.depth);
    let p = &game.player;
    let hunger_label = match p.hunger {
        0 => "STARVING",
        1..=200 => "hungry",
        201..=600 => "ok",
        _ => "full",
    };
    let text = format!(
        " Deep Delve  D{} {}  HP {}/{}  EP {}/{}  {}  Turn {}  Gold {} ",
        game.depth,
        theme.name(),
        p.hp(),
        p.max_hp(),
        p.ep,
        p.max_ep,
        hunger_label,
        game.turn,
        p.gold,
    );
    let style = Style::default().fg(Color::White).bg(Color::Black);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(text, style)))
            .block(Block::default().borders(Borders::ALL)),
        area,
    );
}

/// The main area: map + side panel.
fn render_main(f: &mut Frame, game: &Game, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(26)])
        .split(area);
    render_map(f, game, chunks[0]);
    render_panel(f, game, chunks[1]);
}

/// The level map.
fn render_map(f: &mut Frame, game: &Game, area: Rect) {
    let level = &game.level;
    let theme = Theme::for_depth(game.depth);
    let floor_color = to_ratatui(theme.floor_color());
    let wall_color = to_ratatui(theme.wall_color());
    let buffer = f.buffer_mut();

    let ox = area.x as i32;
    let oy = area.y as i32;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let px = ox + x as i32;
            let py = oy + y as i32;
            if px < area.x as i32 || px >= (area.x + area.width) as i32 {
                continue;
            }
            if py < area.y as i32 || py >= (area.y + area.height) as i32 {
                continue;
            }
            let pos = Pos::new(x, y);
            let idx = level.idx(pos);
            let visible = level.visible[idx];
            let seen = level.seen[idx];
            if !seen && !visible {
                continue;
            }
            let tile = level.tiles[idx];
            let (glyph, color) = tile_glyph(tile, floor_color, wall_color);
            let style = if visible {
                Style::default().fg(color)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            buffer.set_string(px as u16, py as u16, glyph.to_string(), style);

            // Items (only if visible).
            if visible {
                if let Some(item_id) = level.items[idx]
                    && let Some(def) = crate::items::item::ItemDef::by_id(item_id)
                {
                    buffer.set_string(
                        px as u16,
                        py as u16,
                        def.glyph.to_string(),
                        Style::default().fg(to_ratatui(def.color)),
                    );
                }
                // NPCs (only if visible).
                for npc in &level.npcs {
                    if npc.pos == pos {
                        buffer.set_string(
                            px as u16,
                            py as u16,
                            npc.glyph.to_string(),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                }
                // Monsters (only if visible and alive).
                for m in &game.monsters {
                    if m.is_alive() && m.pos() == pos {
                        let def = m.def();
                        buffer.set_string(
                            px as u16,
                            py as u16,
                            def.glyph.to_string(),
                            Style::default().fg(to_ratatui(def.color)),
                        );
                    }
                }
            }
        }
    }

    // The player (always drawn last, on top).
    let ppos = game.player.pos();
    let px = ox + ppos.x as i32;
    let py = oy + ppos.y as i32;
    if px >= area.x as i32
        && px < (area.x + area.width) as i32
        && py >= area.y as i32
        && py < (area.y + area.height) as i32
    {
        buffer.set_string(
            px as u16,
            py as u16,
            "@",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    }
}

/// The glyph and color for a tile.
fn tile_glyph(tile: Tile, floor_color: Color, wall_color: Color) -> (char, Color) {
    match tile {
        Tile::Wall => ('#', wall_color),
        Tile::Floor => ('.', floor_color),
        Tile::DoorClosed => ('+', to_ratatui(CoreColor::Brown)),
        Tile::DoorOpen => ('·', to_ratatui(CoreColor::Brown)),
        Tile::DoorLocked => ('X', to_ratatui(CoreColor::Red)),
        Tile::Water => ('~', to_ratatui(CoreColor::Blue)),
        Tile::Lava => ('*', to_ratatui(CoreColor::Orange)),
        Tile::SporeGas => ('^', to_ratatui(CoreColor::Green)),
        Tile::StairsDown => ('>', to_ratatui(CoreColor::White)),
        Tile::StairsUp => ('<', to_ratatui(CoreColor::White)),
    }
}

/// The side panel: character stats, equipment, inventory, quests.
fn render_panel(f: &mut Frame, game: &Game, area: Rect) {
    let p = &game.player;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        format!("{} {}", p.race.name(), p.class.name()),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("Level {}  ({} XP)", p.level, p.xp),
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(Span::raw(" ")));
    lines.push(Line::from(Span::styled(
        format!(
            "STR {}  DEX {}  CON {}",
            p.attributes.str, p.attributes.dex, p.attributes.con
        ),
        Style::default().fg(Color::Yellow),
    )));
    lines.push(Line::from(Span::styled(
        format!(
            "INT {}  WIS {}  CHA {}",
            p.attributes.int, p.attributes.wis, p.attributes.cha
        ),
        Style::default().fg(Color::Yellow),
    )));
    lines.push(Line::from(Span::raw(" ")));
    lines.push(Line::from(Span::styled(
        format!("AC {}  Attack {}", p.entity.ac, p.entity.attack),
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(Span::styled(
        format!("Weight {}/{}", p.carrying_weight(), p.carrying_capacity()),
        Style::default().fg(Color::Gray),
    )));

    // Statuses.
    let statuses: Vec<String> = p
        .statuses
        .all()
        .iter()
        .map(|ts| ts.status.name().to_string())
        .collect();
    if !statuses.is_empty() {
        lines.push(Line::from(Span::raw(" ")));
        lines.push(Line::from(Span::styled(
            statuses.join(", "),
            Style::default().fg(Color::Magenta),
        )));
    }

    // Equipment.
    lines.push(Line::from(Span::raw(" ")));
    lines.push(Line::from(Span::styled(
        "Equipment:",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        equip_line(p, "weapon"),
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(Span::styled(
        equip_line(p, "armor"),
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(Span::styled(
        equip_line(p, "ring"),
        Style::default().fg(Color::Gray),
    )));

    // Inventory.
    lines.push(Line::from(Span::raw(" ")));
    lines.push(Line::from(Span::styled(
        "Inventory:",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    if p.inventory.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (empty)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, item) in p.inventory.iter().take(12).enumerate() {
            let marker = if p.equipment.is_equipped(i) { "*" } else { " " };
            lines.push(Line::from(Span::styled(
                format!("{}{} {}", marker, i, item.name()),
                Style::default().fg(Color::Gray),
            )));
        }
        if p.inventory.len() > 12 {
            lines.push(Line::from(Span::styled(
                format!("  +{} more", p.inventory.len() - 12),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    // Quests.
    let active = game.quests.active_count();
    let done = game.quests.completed_count();
    lines.push(Line::from(Span::raw(" ")));
    lines.push(Line::from(Span::styled(
        format!("Quests: {} active, {} done", active, done),
        Style::default().fg(Color::Cyan),
    )));

    let block = Block::default().borders(Borders::ALL).title(" Status ");
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

/// A single equipment line.
fn equip_line(p: &crate::entities::player::Player, kind: &str) -> String {
    let idx = match kind {
        "weapon" => p.equipment.weapon,
        "armor" => p.equipment.armor,
        _ => p.equipment.ring_left.or(p.equipment.ring_right),
    };
    match idx {
        Some(i) => format!("  {}: {}", kind, p.inventory[i].name()),
        None => format!("  {}: (none)", kind),
    }
}

/// The message log.
fn render_messages(f: &mut Frame, game: &Game, area: Rect) {
    let recent = game.messages.recent(3);
    let lines: Vec<Line> = recent
        .iter()
        .map(|m| {
            let color = match m.severity {
                Severity::Normal => Color::Gray,
                Severity::Good => Color::Green,
                Severity::Bad => Color::Red,
                Severity::System => Color::Cyan,
                Severity::Magic => Color::Magenta,
            };
            Line::from(Span::styled(m.text.clone(), Style::default().fg(color)))
        })
        .collect();
    let block = Block::default().borders(Borders::ALL).title(" Messages ");
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_glyph_covers_all_tiles() {
        let _ = tile_glyph(Tile::Wall, Color::White, Color::Black);
        let _ = tile_glyph(Tile::Floor, Color::White, Color::Black);
        let _ = tile_glyph(Tile::StairsDown, Color::White, Color::Black);
        let _ = tile_glyph(Tile::Lava, Color::White, Color::Black);
    }
}
