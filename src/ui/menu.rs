//! Menu screens: title, character creation, death, victory.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::data::classes::ClassId;
use crate::data::races::RaceId;
use crate::ui::app::Creation;

pub fn render_title(frame: &mut Frame, has_save: bool) {
    let area = frame.area();
    let lines = vec![
        Line::from(vec![Span::styled(
            "  DEEPDELVE  ",
            Style::default().fg(Color::Yellow).bg(Color::Black).bold(),
        )]),
        Line::from(Span::raw("  A terminal roguelike in Rust")),
        Line::from(Span::raw("")),
        Line::from(Span::raw("  Find the Amulet of the Abyss on depth 25.")),
        Line::from(Span::raw("  Descend. Survive. Escape with your life.")),
        Line::from(Span::raw("")),
        Line::from(Span::raw(
            if has_save {
                "  [N]ew game   [L]oad   [Q]uit"
            } else {
                "  [N]ew game   [Q]uit"
            },
        )),
    ];
    let para = Paragraph::new(lines);
    frame.render_widget(para, Rect {
        x: area.x,
        y: area.y + area.height / 4,
        width: area.width,
        height: (area.height / 2).min(area.height),
    });
}

pub fn render_creation(frame: &mut Frame, creation: &Creation) {
    let area = frame.area();
    let races: Vec<String> = RaceId::ALL
        .iter()
        .map(|r| format!("  {} — {}", r.name(), r.desc()))
        .collect();
    let classes: Vec<String> = ClassId::ALL
        .iter()
        .map(|c| format!("  {} — {}", c.name(), c.desc()))
        .collect();

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "  CHARACTER CREATION",
            Style::default().fg(Color::Yellow).bold(),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled("  Name: ", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(
            format!("    {}", creation.name),
            Style::default().fg(Color::White).underlined(),
        )),
        Line::from(Span::raw("")),
    ];
    lines.push(Line::from(Span::styled("  RACES", Style::default().fg(Color::Green))));
    for r in races {
        lines.push(Line::from(Span::raw(r)));
    }
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled("  CLASSES", Style::default().fg(Color::Green))));
    for c in classes {
        lines.push(Line::from(Span::raw(c)));
    }
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::raw("  [Enter] confirm   [Esc] back")));

    let para = Paragraph::new(lines);
    frame.render_widget(
        para,
        Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height,
        },
    );
}

pub fn render_death(frame: &mut Frame, score: u64, depth: u8, level: u8) {
    let area = frame.area();
    let lines = vec![
        Line::from(Span::styled(
            "  YOU HAVE DIED",
            Style::default().fg(Color::Red).bold(),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::raw(format!("  You died on depth {depth}, level {level}."))),
        Line::from(Span::raw(format!("  Score: {score}"))),
        Line::from(Span::raw("")),
        Line::from(Span::raw("  [Enter] return to title")),
    ];
    let para = Paragraph::new(lines);
    frame.render_widget(
        para,
        Rect {
            x: area.x,
            y: area.y + area.height / 4,
            width: area.width,
            height: (area.height / 2).min(area.height),
        },
    );
}

pub fn render_victory(frame: &mut Frame, score: u64, turns: u64) {
    let area = frame.area();
    let lines = vec![
        Line::from(Span::styled(
            "  VICTORY!",
            Style::default().fg(Color::Yellow).bg(Color::Black).bold(),
        )),
        Line::from(Span::raw("  You raised the Amulet of the Abyss!")),
        Line::from(Span::raw("")),
        Line::from(Span::raw(format!("  Score: {score}   Turns: {turns}"))),
        Line::from(Span::raw("")),
        Line::from(Span::raw("  [Enter] return to title")),
    ];
    let para = Paragraph::new(lines);
    frame.render_widget(
        para,
        Rect {
            x: area.x,
            y: area.y + area.height / 4,
            width: area.width,
            height: (area.height / 2).min(area.height),
        },
    );
}
