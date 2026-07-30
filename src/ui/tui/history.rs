//! TUI download history page.

use super::*;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::base_system::download_history::{DownloadHistoryRecord, read_download_history};

pub(super) fn show_history_menu(app: &mut App) -> Result<()> {
    app.view = View::History;
    refresh_history(app);
    Ok(())
}

pub(super) fn handle_event_history(app: &mut App, event: Event) -> Result<()> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('b') | KeyCode::Esc => {
                app.view = View::Home;
                app.status = "На главную".to_string();
            }
            KeyCode::Char('r') => {
                refresh_history(app);
            }
            KeyCode::Up => select_prev(app),
            KeyCode::Down => select_next(app),
            _ => {}
        },
        Event::Mouse(me) => handle_mouse_history(app, me),
        _ => {}
    }
    Ok(())
}

fn refresh_history(app: &mut App) {
    app.history_entries = read_download_history(200, None);
    if app.history_entries.is_empty() {
        app.history_state.select(None);
        app.status = "История загрузок пуста".to_string();
    } else {
        app.history_state.select(Some(0));
        app.status = format!("Загружена история: {} записей", app.history_entries.len());
    }
}

fn select_next(app: &mut App) {
    if app.history_entries.is_empty() {
        app.history_state.select(None);
        return;
    }
    let next = app
        .history_state
        .selected()
        .map(|i| (i + 1) % app.history_entries.len())
        .unwrap_or(0);
    app.history_state.select(Some(next));
}

fn select_prev(app: &mut App) {
    if app.history_entries.is_empty() {
        app.history_state.select(None);
        return;
    }
    let prev = app
        .history_state
        .selected()
        .map(|i| {
            if i == 0 {
                app.history_entries.len() - 1
            } else {
                i - 1
            }
        })
        .unwrap_or(0);
    app.history_state.select(Some(prev));
}

fn handle_mouse_history(app: &mut App, me: event::MouseEvent) {
    let Some(layout) = app.last_history_layout else {
        return;
    };
    let list_area = layout[1];
    let back_btn = back_button_rect(layout[2]);

    match me.kind {
        MouseEventKind::ScrollUp => select_prev(app),
        MouseEventKind::ScrollDown => select_next(app),
        MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Moved => {
            if matches!(me.kind, MouseEventKind::Down(MouseButton::Left))
                && super::pos_in(back_btn, me.column, me.row)
            {
                app.view = View::Home;
                app.status = "На главную".to_string();
                return;
            }
            if super::pos_in(list_area, me.column, me.row)
                && let Some(idx) = super::list_index_from_mouse_row(
                    list_area,
                    me.row,
                    &app.history_state,
                    app.history_entries.len(),
                )
            {
                app.history_state.select(Some(idx));
            }
        }
        _ => {}
    }
}

pub(super) fn draw_history(frame: &mut ratatui::Frame, app: &mut App) {
    let (main, log_area) = super::split_with_log(frame.size());
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(8),
        ])
        .split(main);

    if layout.len() == 3 {
        let mut arr = [Rect::default(); 3];
        arr.copy_from_slice(&layout);
        app.last_history_layout = Some(arr);
    }

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "История загрузок",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  ↑↓ выбор  r обновить  b назад"),
    ]))
    .block(Block::default().borders(Borders::ALL).title("History"));
    frame.render_widget(header, layout[0]);

    let items: Vec<ListItem> = if app.history_entries.is_empty() {
        vec![ListItem::new("Нет записей")]
    } else {
        app.history_entries
            .iter()
            .map(|it| {
                let st = if it.status.eq_ignore_ascii_case("success") {
                    "успех"
                } else {
                    "ошибка"
                };
                ListItem::new(format!(
                    "[{}] «{}» ({}) | {} | {}",
                    it.timestamp, it.book_name, it.book_id, it.progress, st
                ))
            })
            .collect()
    };

    let list_block = Block::default().borders(Borders::ALL).title("Список записей");
    frame.render_widget(list_block.clone(), layout[1]);
    let inner = list_block.inner(layout[1]);
    let need_scrollbar = app.history_entries.len() > inner.height as usize && inner.width > 1;
    let (list_area, sb_area) = if need_scrollbar {
        let lw = inner.width.saturating_sub(1).max(1);
        (
            Rect {
                x: inner.x,
                y: inner.y,
                width: lw,
                height: inner.height,
            },
            Some(Rect {
                x: inner.x.saturating_add(lw),
                y: inner.y,
                width: 1,
                height: inner.height,
            }),
        )
    } else {
        (inner, None)
    };

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, list_area, &mut app.history_state);

    if let Some(sb) = sb_area {
        let pos = app
            .history_state
            .selected()
            .unwrap_or(0)
            .min(app.history_entries.len().saturating_sub(1));
        let mut s = ScrollbarState::new(app.history_entries.len()).position(pos);
        frame.render_stateful_widget(
            Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight),
            sb,
            &mut s,
        );
    }

    let details = selected_details(app.history_entries.as_slice(), app.history_state.selected());
    let detail_widget = Paragraph::new(details)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Подробности"));
    frame.render_widget(detail_widget, layout[2]);

    let btn_area = back_button_rect(layout[2]);
    let btn = Paragraph::new("[ Назад ]")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(btn, btn_area);

    super::render_log_box(frame, log_area, app);
}

fn back_button_rect(detail_area: Rect) -> Rect {
    let inner = super::list_inner_area(detail_area);
    let w: u16 = 12;
    let h: u16 = 1;
    Rect {
        x: inner.x.saturating_add(inner.width.saturating_sub(w)),
        y: inner.y.saturating_add(inner.height.saturating_sub(h)),
        width: w.min(inner.width.max(1)),
        height: h,
    }
}

fn selected_details(
    items: &[DownloadHistoryRecord],
    selected: Option<usize>,
) -> Vec<Line<'static>> {
    let Some(idx) = selected else {
        return vec![Line::from("Нет данных для отображения")];
    };
    let Some(it) = items.get(idx) else {
        return vec![Line::from("Нет данных для отображения")];
    };

    vec![
        Line::from(format!("Время: {}", it.timestamp)),
        Line::from(format!("Название: {}", it.book_name)),
        Line::from(format!(
            "Автор: {}",
            if it.author.is_empty() {
                "неизвестно"
            } else {
                it.author.as_str()
            }
        )),
        Line::from(format!("Book ID: {}", it.book_id)),
        Line::from(format!("Прогресс: {}", it.progress)),
        Line::from(format!("Статус: {}", it.status)),
    ]
}
