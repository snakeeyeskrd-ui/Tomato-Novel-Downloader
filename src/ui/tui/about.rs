//! TUI About page.
//!
//! Shows project info and buttons (open links, etc.).

use super::*;

pub(super) fn handle_event_about(app: &mut App, event: Event) -> Result<()> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                app.view = View::Home;
                app.status = "Вернуться в главное меню".to_string();
            }
            KeyCode::Enter => match app.about_btn_state.selected() {
                Some(0) => {
                    open_github_repo(app)?;
                }
                Some(1) => {
                    if cfg!(feature = "docker") {
                        app.view = View::Home;
                        app.status = "Вернуться в главное меню".to_string();
                    } else {
                        check_app_update(app)?;
                    }
                }
                Some(2) => {
                    if !cfg!(feature = "docker") {
                        request_self_update(app)?;
                    }
                }
                Some(3) => {
                    if !cfg!(feature = "docker") {
                        dismiss_app_update(app)?;
                    }
                }
                Some(4) => {
                    if !cfg!(feature = "docker") {
                        app.view = View::Home;
                        app.status = "Вернуться в главное меню".to_string();
                    }
                }
                _ => {}
            },
            KeyCode::Up => {
                let idx = app.about_btn_state.selected().unwrap_or(0);
                let prev = if idx == 0 {
                    ABOUT_BUTTONS.len() - 1
                } else {
                    idx - 1
                };
                app.about_btn_state.select(Some(prev));
            }
            KeyCode::Down => {
                let idx = app.about_btn_state.selected().unwrap_or(0);
                let next = (idx + 1) % ABOUT_BUTTONS.len();
                app.about_btn_state.select(Some(next));
            }
            _ => {}
        },
        Event::Mouse(me) => handle_mouse_about(app, me)?,
        _ => {}
    }
    Ok(())
}

pub(super) fn draw_about(frame: &mut ratatui::Frame, app: &mut App) {
    let (main, log_area) = super::split_with_log(frame.size());
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Min(5),
        ])
        .split(main);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "О программе / About",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  q/Esc назад"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Tomato Novel Downloader"),
    );
    frame.render_widget(header, layout[0]);

    let button_area = layout[1];
    let btn_items: Vec<ListItem> = ABOUT_BUTTONS.iter().map(|b| ListItem::new(*b)).collect();
    let btn_list = List::new(btn_items)
        .block(Block::default().borders(Borders::ALL).title("Действия"))
        .highlight_style(
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(btn_list, button_area, &mut app.about_btn_state);
    app.last_about_buttons = Some(button_area);

    let mut text = String::new();
    text.push_str("Репозиторий: https://github.com/zhongbai2333/Tomato-Novel-Downloader\n");
    text.push_str("Fork From: https://github.com/Dlmily/Tomato-Novel-Downloader-Lite\n");
    text.push_str("Автор: zhongbai2333\n");
    text.push_str("Проект предназначен только для обучения и обмена опытом. Не используйте в коммерческих или незаконных целях.\n");
    text.push_str(&format!("\nТекущая версия: v{}\n", env!("CARGO_PKG_VERSION")));

    #[cfg(feature = "official-api")]
    {
        text.push_str("\n===== Бесплатность =====\n");
        text.push_str(
            "Программа полностью бесплатна. Официальный репозиторий: https://github.com/zhongbai2333/Tomato-Novel-Downloader\n",
        );
        text.push_str("Если вы за неё заплатили — вас обманули.\n");
    }

    text.push_str("\n===== Обновления =====\n");
    if cfg!(feature = "docker") {
        text.push_str("В Docker-сборке автообновление отключено. Обновите образ (docker pull).\n");
    } else if let Some(rep) = &app.app_update_report {
        text.push_str(&format!("Текущая: {}\n", rep.current_tag));
        text.push_str(&format!("Последняя: {}\n", rep.latest.tag_name));
        if rep.is_new_version {
            text.push_str("Статус: есть новая версия\n");
        } else {
            text.push_str("Статус: уже актуальная версия\n");
        }
        if rep.is_dismissed {
            text.push_str("Подсказка: напоминание об этой версии отключено (ручная проверка всё ещё доступна)\n");
        }
        if let Some(url) = rep.latest.html_url.as_deref()
            && !url.trim().is_empty()
        {
            text.push_str(&format!("Release: {}\n", url));
        }
        if let Some(body) = rep.latest.body.as_deref() {
            let body = body.trim();
            if !body.is_empty() {
                text.push_str("\nСписок изменений (фрагмент):\n");
                text.push_str(&preview_notes(body, 16, 1800));
                text.push('\n');
            }
        }
    } else {
        text.push_str("Обновления не проверялись (нажмите «Проверить обновления»)\n");
    }

    let body = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("О проекте"));
    frame.render_widget(body, layout[2]);

    super::render_log_box(frame, log_area, app);
}

fn check_app_update(app: &mut App) -> Result<()> {
    app.status = "Проверка обновлений…".to_string();
    super::start_app_update_check(app);
    Ok(())
}

fn dismiss_app_update(app: &mut App) -> Result<()> {
    let Some(rep) = app.app_update_report.clone() else {
        app.status = "Сведения об обновлении ещё не получены — сначала нажмите «Проверить обновления»".to_string();
        return Ok(());
    };
    if !rep.is_new_version {
        app.status = "Уже актуальная версия, напоминание не нужно".to_string();
        return Ok(());
    }
    let tag = rep.latest.tag_name.clone();
    crate::base_system::app_update::dismiss_release_tag(&tag)?;
    let mut new_rep = rep;
    new_rep.is_dismissed = true;
    app.app_update_report = Some(new_rep);
    app.status = format!("Напоминание отключено для {}", tag);
    Ok(())
}

fn request_self_update(app: &mut App) -> Result<()> {
    app.status = "Сейчас выполнится автообновление (после выхода из TUI)…".to_string();
    app.self_update_requested = true;
    // The TUI already has an explicit "Run self-update" button; clicking it is confirmation.
    // So we do not ask again inside self_update.
    app.self_update_auto_yes = true;
    app.should_quit = true;
    Ok(())
}

fn preview_notes(body: &str, max_lines: usize, max_chars: usize) -> String {
    let mut out = String::new();
    for (i, line) in body.lines().enumerate() {
        if i >= max_lines {
            out.push('…');
            break;
        }
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line);
        if out.len() >= max_chars {
            // Truncate on a char boundary to avoid panicking on multibyte characters
            let mut end = max_chars;
            while !out.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            out.truncate(end);
            out.push('…');
            break;
        }
    }
    out
}

fn handle_mouse_about(app: &mut App, me: event::MouseEvent) -> Result<()> {
    let Some(area) = app.last_about_buttons else {
        return Ok(());
    };
    let pos_in = |rect: Rect, col: u16, row: u16| {
        col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
    };
    if !pos_in(area, me.column, me.row) {
        return Ok(());
    }

    match me.kind {
        MouseEventKind::Moved => {
            let idx = me.row.saturating_sub(area.y + 1) as usize;
            if idx < ABOUT_BUTTONS.len() {
                app.about_btn_state.select(Some(idx));
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            let idx = me.row.saturating_sub(area.y + 1) as usize;
            match idx {
                0 => {
                    app.about_btn_state.select(Some(0));
                    open_github_repo(app)?;
                }
                1 => {
                    app.about_btn_state.select(Some(1));
                    if cfg!(feature = "docker") {
                        app.view = View::Home;
                        app.status = "Вернуться в главное меню".to_string();
                    } else {
                        check_app_update(app)?;
                    }
                }
                2 => {
                    if !cfg!(feature = "docker") {
                        app.about_btn_state.select(Some(2));
                        request_self_update(app)?;
                    }
                }
                3 => {
                    if !cfg!(feature = "docker") {
                        app.about_btn_state.select(Some(3));
                        dismiss_app_update(app)?;
                    }
                }
                4 => {
                    if !cfg!(feature = "docker") {
                        app.about_btn_state.select(Some(4));
                        app.view = View::Home;
                        app.status = "Вернуться в главное меню".to_string();
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    Ok(())
}

fn open_github_repo(app: &mut App) -> Result<()> {
    let url = "https://github.com/zhongbai2333/Tomato-Novel-Downloader";
    let spawn_result = if cfg!(target_os = "windows") {
        // Avoid spawning `cmd.exe` (it can mutate console modes and break mouse events).
        // `explorer.exe` uses the default URL handler without touching our console settings.
        Command::new("explorer")
            .arg(url)
            .spawn()
            .or_else(|_| Command::new("cmd").args(["/C", "start", url]).spawn())
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(url).spawn()
    } else {
        Command::new("xdg-open").arg(url).spawn()
    };

    match spawn_result {
        Ok(_) => app.status = format!("Попытка открыть в браузере: {url}"),
        Err(e) => app.status = format!("Не удалось открыть браузер: {e}"),
    }

    // Best-effort: some OS openers may still toggle console modes.
    // Re-assert our expected modes so returning to the TUI keeps mouse usable.
    let _ = enable_raw_mode();
    let mut out = std::io::stdout();
    let _ = crossterm_execute!(&mut out, EnableMouseCapture);
    Ok(())
}
