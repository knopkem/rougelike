//! Rendering: main frame, viewport, bars, status, messages.

use ratatui::{
     Frame,
     buffer::Cell,
     layout::{Constraint, Direction, Layout, Rect},
     style::{Color, Style},
     text::{Line, Span},
     widgets::{Gauge, Paragraph},
 };

use crate::core::game::Game;
use crate::map::level::{Level, LevelTheme, MAP_H, MAP_W};
use crate::ui::palette;

/// Render the main play frame.
pub fn render(frame: &mut Frame, game: &Game) {
    let size = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(18),
            Constraint::Length(4),
            Constraint::Length(2),
        ])
        .split(size);

    render_viewport(frame, game, chunks[0]);
    render_status(frame, game, chunks[1]);
    render_messages(frame, game, chunks[2]);
}

fn render_viewport(frame: &mut Frame, game: &Game, area: Rect) {
    let level = game.current();
    let theme = level.theme;
    let (ox, oy) = (area.x, area.y);

    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let pos = (x, y);
            let screen_x = ox + x as u16;
            let screen_y = oy + y as u16;
            if screen_x >= area.x + area.width || screen_y >= area.y + area.height {
                continue;
            }
            let tile = level.tile_at(pos);
            let seen = level.seen[Level::pos_idx(pos)];
            let explored = level.explored[Level::pos_idx(pos)];
            if !explored {
                continue;
            }
            let (ch, fg, dim) = if pos == game.player.pos {
                ("@", Color::White, false)
            } else {
                match tile {
                    crate::map::level::Tile::Wall => ("#", palette::wall(theme), !seen),
                    crate::map::level::Tile::Floor => (".", palette::floor(theme), !seen),
                    crate::map::level::Tile::StairsDown => (">", Color::Yellow, !seen),
                    crate::map::level::Tile::StairsUp => ("<", Color::Yellow, !seen),
                    crate::map::level::Tile::Water => ("~", Color::Blue, !seen),
                    crate::map::level::Tile::Lava => (
                        "*",
                        Color::LightRed,
                        !seen,
                    ),
                    _ => (".", Color::Gray, !seen),
                }
            };
            let style = if dim {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(fg)
            };
            frame.buffer_mut().set_string(screen_x, screen_y, ch, style);
        }
    }

    // Monsters in FOV.
    for m in &game.monsters {
          if level.seen[Level::pos_idx(m.pos)] {
            let style = Style::default().fg(Color::LightRed);
            let glyph = m.def.glyph.to_string();
            frame.buffer_mut()
                .set_string(ox + m.pos.0 as u16, oy + m.pos.1 as u16, glyph, style);
        }
    }

    // Gold piles (in FOV).
    for (i, _) in &level.gold {
        let x = (*i % MAP_W as usize) as u8;
        let y = (*i / MAP_W as usize) as u8;
        if level.seen[Level::pos_idx((x, y))] && (x, y) != game.player.pos {
            let style = Style::default().fg(Color::Yellow);
            frame.buffer_mut()
                .set_string(ox + x as u16, oy + y as u16, "$", style);
        }
    }

    // Items on ground (in FOV).
    for (i, items) in &level.items {
        let x = (*i % MAP_W as usize) as u8;
        let y = (*i / MAP_W as usize) as u8;
        if level.seen[Level::pos_idx((x, y))]
            && !items.is_empty()
            && (x, y) != game.player.pos
        {
            let glyph = items.first().map(|it| it.glyph()).unwrap_or('?');
            let style = Style::default().fg(Color::Cyan);
            frame.buffer_mut()
                .set_string(
                    ox + x as u16,
                    oy + y as u16,
                    glyph.to_string(),
                    style,
                );
        }
    }
}

fn render_status(frame: &mut Frame, game: &Game, area: Rect) {
    let p = &game.player;

    // HP gauge.
    let hp_area = Rect {
        x: area.x,
        y: area.y,
        width: 30,
        height: 1,
    };
    let hp_pct = if p.max_hp > 0 {
        (p.hp as f64 / p.max_hp as f64) * 100.0
    } else {
        0.0
    };
    let hp_color = if hp_pct > 50.0 {
        Color::Green
    } else if hp_pct > 25.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    let hp_gauge = Gauge::default()
        .ratio(hp_pct / 100.0)
        .gauge_style(Style::default().fg(Color::White).bg(hp_color))
         .label(format!("HP {}/{}", p.hp, p.max_hp));
    frame.render_widget(hp_gauge, hp_area);

    // EP gauge.
    let ep_area = Rect {
        x: area.x + 32,
        y: area.y,
        width: 20,
        height: 1,
    };
    let ep_pct = if p.max_ep > 0 {
        (p.ep as f64 / p.max_ep as f64) * 100.0
    } else {
        0.0
    };
    let ep_gauge = Gauge::default()
        .ratio(ep_pct / 100.0)
        .gauge_style(Style::default().fg(Color::White).bg(Color::Blue))
        .label(format!("EP {}/{}", p.ep, p.max_ep));
    frame.render_widget(ep_gauge, ep_area);

    // Hunger bar.
    let hunger_area = Rect {
        x: area.x + 54,
        y: area.y,
        width: 22,
        height: 1,
    };
    let hunger_pct = p.hunger as f64 / 1200.0 * 100.0;
    let hunger_color = if p.hunger > 300 {
        Color::Yellow
    } else {
        Color::Red
    };
    let hunger_gauge = Gauge::default()
        .ratio(hunger_pct / 100.0)
        .gauge_style(Style::default().fg(Color::Black).bg(hunger_color))
        .label(format!("FU {}/1200", p.hunger));
    frame.render_widget(hunger_gauge, hunger_area);

    // Second line: name, class, depth, gold.
    let line = format!(
        "@ {} {} Lv{} {}  D{}: {}  ${}",
        p.name,
        p.class,
        p.level,
        p.race,
        game.current_level,
        crate::data::themes::zone_name(game.current_level),
        p.gold,
    );
    let para = Paragraph::new(Line::from(Span::raw(line))).style(Style::default().fg(Color::White));
    frame.render_widget(
        para,
        Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: 1,
        },
    );
}

fn render_messages(frame: &mut Frame, game: &Game, area: Rect) {
    let msgs = game.messages.tail(2);
    let lines: Vec<Line> = msgs
        .iter()
        .map(|m| Line::from(Span::raw(format!("> {}", m.text))))
        .collect();
    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}
