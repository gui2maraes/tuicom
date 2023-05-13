use crate::app::{App, Mode};
use itertools::Itertools;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::Clear,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(48),
            Constraint::Percentage(48),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.size());

    // tx
    draw_tx(f, app, chunks[0]);

    // rx
    draw_rx(f, app, chunks[1]);

    // bindings
    draw_bindings(f, chunks[2]);

    // status line
    draw_status(f, app, chunks[3]);

    match &app.mode {
        Mode::WannaQuit => draw_quit_popup(f),
        Mode::BaudInput(s) => draw_baud_popup(f, s),
        _ => (),
    };
}

fn draw_tx<B: Backend>(f: &mut Frame<B>, app: &mut App, rect: Rect) {
    let title = if app.tx.is_ascii() {
        "[TX]"
    } else {
        "[TX - Hex]"
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::all())
        .border_type(if app.mode.is_insert() {
            BorderType::Thick
        } else {
            BorderType::Plain
        });
    let inner = block.inner(rect);
    let tx = app.tx.with_cursor(app.cursor());
    let txt = Paragraph::new(tx.as_ref())
        .block(block)
        .scroll((scroll_amount(tx.as_ref(), inner), 0))
        .wrap(Wrap { trim: false });

    f.render_widget(txt, rect);
}

fn draw_rx<B: Backend>(f: &mut Frame<B>, app: &mut App, rect: Rect) {
    let title = if app.rx.is_ascii() {
        "[RX]"
    } else {
        "[RX - Hex]"
    };
    let block = Block::default().title(title).borders(Borders::all());
    let inner = block.inner(rect);

    let rx = app.rx.with_cursor(app.cursor());
    let txt = Paragraph::new(rx.as_ref())
        .block(block)
        .scroll((scroll_amount(rx.as_ref(), inner), 0))
        .wrap(Wrap { trim: false });
    f.render_widget(txt, rect);
}

fn draw_status<B: Backend>(f: &mut Frame<B>, app: &App, rect: Rect) {
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let port_name = app.serial.name().unwrap_or_else(|| String::from("serial"));

    let baud_rate = app
        .serial
        .baud_rate()
        .map(|b| b.to_string())
        .unwrap_or_else(|_| String::from("<baud>"));
    let mode = if app.mode.is_insert() {
        "INSERT"
    } else {
        "NORMAL"
    };
    let crlf = if app.tx.lf_crlf { "CR + LF" } else { "LF" };
    let spans = Spans::from(vec![
        Span::styled(mode, bold),
        Span::raw(" | "),
        Span::styled(port_name, bold),
        Span::raw(" | "),
        Span::styled(baud_rate, bold),
        Span::raw(" | "),
        Span::styled(crlf, bold),
    ]);
    let p = Paragraph::new(spans).style(Style::default().bg(Color::DarkGray));
    f.render_widget(p, rect);
}

static BINDINGS: &[(&str, &str)] = &[
    ("q", "quit"),
    ("H", "TX hex"),
    ("h", "RX hex"),
    ("C", "clear TX"),
    ("c", "clear RX"),
    ("l", "map LF to CR + LF"),
    ("b", "change baud rate"),
    ("i", "insert mode"),
    ("ESC", "normal mode"),
];
fn draw_bindings<B: Backend>(f: &mut Frame<B>, rect: Rect) {
    let spans = Spans::from(
        BINDINGS
            .iter()
            .map(|(key, action)| {
                [
                    Span::styled(*key, Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(": "),
                    Span::raw(*action),
                ]
            })
            .intersperse([Span::raw(" "), Span::raw("|"), Span::raw(" ")])
            .flatten()
            .collect::<Vec<_>>(),
    );
    let p = Paragraph::new(spans).style(Style::default().bg(Color::DarkGray));
    f.render_widget(p, rect);
}

fn draw_quit_popup<B: Backend>(f: &mut Frame<B>) {
    let block = Block::default().title("Quit").borders(Borders::all());
    let area = centered_rect(30, 20, f.size());
    let txt = Paragraph::new("Are you sure you want to quit (y/n)?")
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(Clear, area);
    f.render_widget(txt, area);
}

fn draw_baud_popup<B: Backend>(f: &mut Frame<B>, baud_input: &str) {
    use ratatui::symbols::block::SEVEN_EIGHTHS as cursor;
    let block = Block::default().title("Baud Rate").borders(Borders::all());
    let area = centered_rect(20, 20, f.size());
    let spans = Spans::from(vec![
        Span::raw("Baud rate: "),
        Span::raw(baud_input),
        Span::styled(cursor, Style::default().add_modifier(Modifier::SLOW_BLINK)),
    ]);
    let txt = Paragraph::new(spans).block(block).wrap(Wrap { trim: true });
    f.render_widget(Clear, area);
    f.render_widget(txt, area);
}

// functions for autoscrolling the text areas
fn scroll_amount(s: &str, area: Rect) -> u16 {
    let lines = lines(s, area);
    if lines > area.height {
        lines - area.height
    } else {
        0
    }
}
fn lines(s: &str, area: Rect) -> u16 {
    let mut line_count = 0;
    for line in s.lines() {
        line_count += 1;
        let mut line_len = line.len();
        while line_len > area.width as usize {
            line_count += 1;
            line_len -= area.width as usize;
        }
    }
    line_count as u16
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
