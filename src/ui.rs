use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph, Wrap,
    },
    Frame,
};
use tui_markdown::from_str;

use crate::{
    app::{App, InputMode},
    screen::Screen,
};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    match app.screen {
        Screen::Home => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(3)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(frame.area());

            render_header(
                app,
                frame,
                chunks[0],
                "Choose a feed or enter a new one to get started.",
            );
            render_help_message(app, frame, chunks[0]);
            render_input_field(app, frame, chunks[1]);
            render_feed_list(app, frame, chunks[2]);

            if app.confirmation_popup.is_some() {
                render_confirmation_popup(app, frame, frame.area());
            }
        }
        Screen::Feed => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(3)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(0),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(frame.area());

            // Replace with feed content
            render_header(app, frame, chunks[0], "Choose an article to read.");
            render_article_list(app, frame, chunks[2]);
        }
        Screen::Article => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(3)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(0),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(frame.area());

            render_header(app, frame, chunks[0], app.current_entry.title.as_str());
            render_article(app, frame, chunks[2]);
        }
    }
}

fn render_header(_: &App, frame: &mut Frame, _: Rect, text: &str) {
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::bordered()
                    .title("Fead")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .centered(),
        frame.area(),
    )
}

fn render_help_message(app: &App, frame: &mut Frame, area: Rect) {
    let (data, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("CTRL-C", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("Escape", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to submit the feed url."),
            ],
            Style::default().fg(Color::Yellow),
        ),
    };

    let text = Text::from(Line::from(data)).style(style);
    let help_message = Paragraph::new(text);

    frame.render_widget(help_message, area);
}

fn render_input_field(app: &App, frame: &mut Frame, area: Rect) {
    let width = area.width.max(3) - 3;
    let scroll = app.input.visual_scroll(width as usize);

    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    frame.render_widget(input, area);

    if let InputMode::Editing = app.input_mode {
        frame.set_cursor_position((
            area.x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            area.y + 1,
        ));
    }
}

fn render_feed_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app
        .feed_list
        .items
        .iter()
        .map(|feed| ListItem::from(feed.title.clone()).fg(Color::Cyan))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Feeds")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, area, &mut app.feed_list.state);
}

fn render_article_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app
        .entry_list
        .items
        .iter()
        .map(|entry| ListItem::from(entry.title.clone()).fg(Color::Cyan))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Articles")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(list, area, &mut app.entry_list.state);
}

fn render_article(app: &mut App, frame: &mut Frame, area: Rect) {
    let data = app.current_entry.content.as_str();

    let binding = htmd::convert(data).unwrap();
    let text = from_str(binding.as_str());

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true })
        .scroll((app.scroll_offset, 0));

    frame.render_widget(paragraph, area);
}

fn render_confirmation_popup(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(popup) = &app.confirmation_popup {
        let block = Block::default()
            .title_alignment(Alignment::Center)
            .title(popup.message.as_str())
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Red).bg(Color::Black));

        let popup_area = centered_rect(60, 25, area);

        frame.render_widget(Clear, popup_area);
        frame.render_widget(block, popup_area);

        let button_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
            .margin(1)
            .split(popup_area);

        let button_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(button_layout[1]);

        let no_button_style = if popup.selected_button == 0 {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };

        let yes_button_style = if popup.selected_button == 1 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let no_button = Paragraph::new("No")
            .block(Block::default().borders(Borders::ALL))
            .style(no_button_style)
            .alignment(Alignment::Center);

        let yes_button = Paragraph::new("Yes")
            .block(Block::default().borders(Borders::ALL))
            .style(yes_button_style)
            .alignment(Alignment::Center);

        frame.render_widget(no_button, button_row[0]);
        frame.render_widget(yes_button, button_row[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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
        .split(area);

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
