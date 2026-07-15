use ratatui::widgets::ListState;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph, Wrap,
    },
    Frame,
};
use tui_markdown::from_str;

use crate::{
    app::{App, ConfirmationChoice, InputMode, Notice},
    screen::Screen,
};

const TEXT: Color = Color::Cyan;
const ACCENT: Color = Color::Yellow;
const SURFACE: Color = Color::Black;
const DANGER: Color = Color::Red;
const SUCCESS: Color = Color::Green;

pub fn render(app: &mut App, frame: &mut Frame) {
    frame.render_widget(Block::new().style(Style::new().bg(SURFACE)), frame.area());

    let margin = if frame.area().width < 64 { 1 } else { 3 };
    let area = Layout::default()
        .margin(margin)
        .constraints([Constraint::Min(1)])
        .split(frame.area())[0];

    let sections = match app.screen {
        Screen::Home => Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area),
        Screen::Feed | Screen::Article => Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area),
    };

    render_header(app, frame, sections[0]);
    match app.screen {
        Screen::Home => {
            render_input(app, frame, sections[1]);
            render_feed_list(app, frame, sections[2]);
            render_status(app, frame, sections[3]);
        }
        Screen::Feed => {
            render_article_list(app, frame, sections[1]);
            render_status(app, frame, sections[2]);
        }
        Screen::Article => {
            render_article(app, frame, sections[1]);
            render_status(app, frame, sections[2]);
        }
    }

    if app.confirmation_popup.is_some() {
        render_confirmation(app, frame);
    }
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let context = match app.screen {
        Screen::Home => "Choose a feed or enter a new one to get started.",
        Screen::Feed => "Choose an article to read.",
        Screen::Article => app.current_entry.title.as_str(),
    };
    frame.render_widget(
        Paragraph::new(context)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Fead")
                    .title_alignment(Alignment::Center),
            )
            .style(Style::new().fg(TEXT).bg(SURFACE))
            .alignment(Alignment::Center),
        area,
    );
}

fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    let editing = app.input_mode == InputMode::Editing;
    let width = area.width.saturating_sub(3).max(1);
    let scroll = app.input.visual_scroll(width as usize);
    let value = app.input.value();
    let style = if editing {
        Style::new().fg(ACCENT)
    } else {
        Style::new().fg(Color::White)
    };
    let border_style = if editing {
        Style::new().fg(ACCENT)
    } else {
        Style::new().fg(Color::White)
    };

    frame.render_widget(
        Paragraph::new(value)
            .style(style)
            .scroll((0, scroll as u16))
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title("Input"),
            ),
        area,
    );

    if editing {
        frame.set_cursor_position((
            area.x + (app.input.visual_cursor().max(scroll) - scroll) as u16 + 1,
            area.y + 1,
        ));
    }
}

fn render_feed_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let items = app
        .feed_list
        .items
        .iter()
        .map(|feed| ListItem::new(Line::from(feed.title.as_str())))
        .collect::<Vec<_>>();
    render_list(
        frame,
        area,
        items,
        &mut app.feed_list.state,
        "Feeds",
        "No feeds yet",
        "Press a and paste an RSS or Atom URL.",
    );
}

fn render_article_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let items = app
        .entry_list
        .items
        .iter()
        .map(|entry| ListItem::new(Line::from(entry.title.as_str())))
        .collect::<Vec<_>>();
    render_list(
        frame,
        area,
        items,
        &mut app.entry_list.state,
        "Articles",
        "No articles",
        "This feed did not return any entries.",
    );
}

fn render_list(
    frame: &mut Frame,
    area: Rect,
    items: Vec<ListItem<'_>>,
    state: &mut ListState,
    block_title: &'static str,
    empty_title: &str,
    empty_body: &str,
) {
    if items.is_empty() {
        render_empty(frame, area, block_title, empty_title, empty_body);
        return;
    }

    let list = List::new(items)
        .block(list_block(block_title))
        .style(Style::new().fg(TEXT))
        .highlight_style(Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);
    frame.render_stateful_widget(list, area, state);
}

fn render_article(app: &mut App, frame: &mut Frame, area: Rect) {
    let Some(content) = app.current_entry.body().cloned() else {
        frame.render_widget(
            Paragraph::new("This article has no readable content.").style(Style::new().fg(TEXT)),
            area,
        );
        app.update_article_viewport(0, area.height);
        return;
    };
    let markdown;
    let text = if content.kind.is_markup() {
        markdown = htmd::convert(&content.value).unwrap_or_else(|_| content.value.clone());
        from_str(&markdown)
    } else {
        Text::raw(content.value.as_str())
    };
    let line_count = wrapped_line_count(&text, area.width.max(1));
    let paragraph = Paragraph::new(text)
        .style(Style::new().fg(TEXT))
        .wrap(Wrap { trim: true })
        .scroll((app.scroll_offset, 0));

    app.update_article_viewport(line_count, area.height);
    frame.render_widget(paragraph, area);
}

fn render_status(app: &App, frame: &mut Frame, area: Rect) {
    let (text, style) = match &app.notice {
        Some(Notice::Error(message)) => (message.as_str(), Style::new().fg(DANGER)),
        Some(Notice::Info(message)) => (message.as_str(), Style::new().fg(SUCCESS)),
        None if app.input_mode == InputMode::Editing => {
            ("Enter add  ·  Esc cancel", Style::new().fg(ACCENT))
        }
        None => (help_for(app.screen), Style::new().fg(Color::White)),
    };
    frame.render_widget(Paragraph::new(text).style(style), area);
}

fn render_empty(frame: &mut Frame, area: Rect, block_title: &'static str, title: &str, body: &str) {
    let text = vec![
        Line::styled(title, Style::new().fg(TEXT).add_modifier(Modifier::BOLD)),
        Line::styled(body, Style::new().fg(ACCENT)),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(list_block(block_title))
            .alignment(Alignment::Center),
        area,
    );
}

fn render_confirmation(app: &App, frame: &mut Frame) {
    let Some(popup) = &app.confirmation_popup else {
        return;
    };
    let area = centered_fixed(frame.area(), 58, 11);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(DANGER))
            .style(Style::new().bg(SURFACE))
            .title("Delete Feed")
            .title_alignment(Alignment::Center),
        area,
    );

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .split(area);
    frame.render_widget(
        Paragraph::new(popup.message.as_str())
            .style(Style::new().fg(TEXT))
            .alignment(Alignment::Center),
        rows[1],
    );

    let buttons = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(12),
        Constraint::Length(3),
        Constraint::Length(12),
        Constraint::Fill(1),
    ])
    .split(rows[3]);

    let cancel_style = if popup.choice == ConfirmationChoice::Cancel {
        Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(Color::White)
    };
    let delete_style = if popup.choice == ConfirmationChoice::Delete {
        Style::new().fg(DANGER).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(Color::White)
    };
    frame.render_widget(
        Paragraph::new("Cancel")
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(cancel_style),
            )
            .style(cancel_style)
            .alignment(Alignment::Center),
        buttons[1],
    );
    frame.render_widget(
        Paragraph::new("Delete")
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(delete_style),
            )
            .style(delete_style)
            .alignment(Alignment::Center),
        buttons[3],
    );
    frame.render_widget(
        Paragraph::new("←/→ choose  ·  Enter confirm  ·  Esc cancel")
            .style(Style::new().fg(Color::White))
            .alignment(Alignment::Center),
        rows[5],
    );
}

fn list_block(title: &'static str) -> Block<'static> {
    Block::new()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::White))
        .title(title)
        .title_alignment(Alignment::Center)
}

fn help_for(screen: Screen) -> &'static str {
    match screen {
        Screen::Home => "↑/↓ move  ·  Enter open  ·  a add  ·  Backspace delete  ·  q quit",
        Screen::Feed => "↑/↓ move  ·  Enter open  ·  Esc back  ·  q quit",
        Screen::Article => "↑/↓ scroll  ·  PgUp/PgDn page  ·  Esc back  ·  q quit",
    }
}

fn wrapped_line_count(text: &Text<'_>, width: u16) -> usize {
    let width = width as usize;
    text.lines
        .iter()
        .map(|line| line.width().max(1).div_ceil(width))
        .sum()
}

fn centered_fixed(area: Rect, preferred_width: u16, preferred_height: u16) -> Rect {
    let width = preferred_width.min(area.width.saturating_sub(2)).max(1);
    let height = preferred_height.min(area.height.saturating_sub(2)).max(1);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use ratatui::{backend::TestBackend, widgets::ListState, Terminal};
    use tui_input::Input;

    use crate::{
        app::{App, ConfirmationChoice, ConfirmationPopup, EntryList, FeedList, InputMode},
        entries::{ContentKind, Entry, EntryContent},
        feeds::Feed,
        screen::Screen,
    };

    use super::{render, ACCENT};

    #[test]
    fn keeps_the_original_identity_and_spaces_the_delete_dialog() {
        let mut app = test_app();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

        terminal.draw(|frame| render(&mut app, frame)).unwrap();

        {
            let buffer = terminal.backend().buffer();
            let lines = buffer_lines(buffer);

            assert!(lines.iter().any(|line| line.contains("Fead")));
            assert!(lines.iter().any(|line| line.contains("Input")));
            assert!(lines.iter().any(|line| line.contains("Feeds")));

            let selected_row = lines
                .iter()
                .position(|line| line.contains("> Example feed"))
                .unwrap();
            let selected_column = lines[selected_row].find("Example feed").unwrap() as u16;
            assert_eq!(buffer[(selected_column, selected_row as u16)].fg, ACCENT);
        }

        app.confirmation_popup = Some(ConfirmationPopup {
            message: "Delete selected feed?".to_string(),
            choice: ConfirmationChoice::Cancel,
        });
        terminal.draw(|frame| render(&mut app, frame)).unwrap();
        let lines = buffer_lines(terminal.backend().buffer());

        let message_row = lines
            .iter()
            .position(|line| line.contains("Delete selected feed?"))
            .unwrap();
        let button_row = lines
            .iter()
            .position(|line| line.contains("Cancel") && line.contains("Delete"))
            .unwrap();
        let help_row = lines
            .iter()
            .position(|line| line.contains("Enter confirm"))
            .unwrap();

        assert!(button_row >= message_row + 3);
        assert!(help_row >= button_row + 3);
    }

    #[test]
    fn renders_plain_text_without_markdown_interpretation() {
        let mut app = test_app();
        app.screen = Screen::Article;
        app.current_entry = Entry {
            title: "Plain text".to_string(),
            content: Some(EntryContent {
                value: "# literal * text _and_ `code`".to_string(),
                kind: ContentKind::Text,
            }),
            ..Entry::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

        terminal.draw(|frame| render(&mut app, frame)).unwrap();

        let lines = buffer_lines(terminal.backend().buffer());
        assert!(lines
            .iter()
            .any(|line| line.contains("# literal * text _and_ `code`")));
    }

    fn test_app() -> App {
        let mut feed_state = ListState::default();
        feed_state.select_first();
        App {
            running: true,
            screen: Screen::Home,
            input: Input::default(),
            input_mode: InputMode::Normal,
            feed_list: FeedList {
                items: vec![Feed {
                    title: "Example feed".to_string(),
                    url: "https://example.com/feed.xml".to_string(),
                }],
                state: feed_state,
            },
            entry_list: EntryList {
                items: Vec::new(),
                state: ListState::default(),
            },
            current_entry: Entry::default(),
            scroll_offset: 0,
            max_scroll: 0,
            confirmation_popup: None,
            notice: None,
        }
    }

    fn buffer_lines(buffer: &ratatui::buffer::Buffer) -> Vec<String> {
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect()
            })
            .collect()
    }
}
