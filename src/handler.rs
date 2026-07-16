use crate::{
    app::{
        App, AppResult, ConfirmationChoice, ConfirmationKind, ConfirmationPopup, InputMode,
    },
    entries::FeedDocument,
    feeds::FeedsManager,
    fetch::{fetch_content, FetchError},
    reader::parse_feed,
    screen::Screen,
    tts::TTS,
    tts_model::{download_model, model_dir, model_ready, TtsModelEvent},
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;
use tui_input::backend::crossterm::EventHandler;

#[derive(Clone, Copy)]
enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, Copy)]
enum Edge {
    First,
    Last,
}

pub async fn handle_key_events(
    key: KeyEvent,
    app: &mut App,
    model_tx: &mpsc::UnboundedSender<TtsModelEvent>,
) -> AppResult<()> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c' | 'C'))
    {
        app.quit();
        return Ok(());
    }

    if app.confirmation_popup.is_some() {
        handle_confirmation(key, app, model_tx)?;
        return Ok(());
    }

    match app.input_mode {
        InputMode::Normal => handle_normal_mode(key, app).await?,
        InputMode::Editing => handle_editing_mode(key, app).await?,
    }

    Ok(())
}

pub fn handle_tts_model_event(app: &mut App, event: TtsModelEvent) {
    match event {
        TtsModelEvent::Progress { percent } => {
            app.tts_downloading = true;
            app.show_info(format!("Downloading Kokoro EN model… {percent}%"));
        }
        TtsModelEvent::Finished(result) => {
            app.tts_downloading = false;
            match result {
                Ok(()) => match TTS::load() {
                    Ok(tts) => {
                        app.tts = Some(tts);
                        app.show_info("TTS model ready.");
                    }
                    Err(error) => app.show_error(error),
                },
                Err(error) => app.show_error(format!("TTS download failed: {error}")),
            }
        }
    }
}

fn handle_confirmation(
    key: KeyEvent,
    app: &mut App,
    model_tx: &mpsc::UnboundedSender<TtsModelEvent>,
) -> AppResult<()> {
    match key.code {
        KeyCode::Esc => app.confirmation_popup = None,
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::BackTab => {
            if let Some(popup) = &mut app.confirmation_popup {
                popup.choice.toggle();
            }
        }
        KeyCode::Enter => {
            let Some(popup) = app.confirmation_popup.take() else {
                return Ok(());
            };
            if popup.choice != ConfirmationChoice::Accept {
                return Ok(());
            }
            match popup.kind {
                ConfirmationKind::DeleteFeed => delete_selected_feed(app)?,
                ConfirmationKind::DownloadTtsModel => start_tts_download(app, model_tx)?,
            }
        }
        _ => {}
    }
    Ok(())
}

fn start_tts_download(
    app: &mut App,
    model_tx: &mpsc::UnboundedSender<TtsModelEvent>,
) -> AppResult<()> {
    if app.tts_downloading {
        app.show_info("TTS model download already in progress.");
        return Ok(());
    }

    app.tts_downloading = true;
    app.show_info("Downloading Kokoro EN model… 0%");

    let tx = model_tx.clone();
    tokio::spawn(async move {
        let result = download_model(|percent| {
            let _ = tx.send(TtsModelEvent::Progress { percent });
        })
        .await;
        let _ = tx.send(TtsModelEvent::Finished(result));
    });

    Ok(())
}

async fn handle_normal_mode(key: KeyEvent, app: &mut App) -> AppResult<()> {
    match key.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('a' | '/') if app.screen == Screen::Home => {
            app.notice = None;
            app.input_mode = InputMode::Editing;
        }
        KeyCode::Delete | KeyCode::Backspace if app.screen == Screen::Home => {
            if let Some(selected) = app.feed_list.state.selected() {
                let title = &app.feed_list.items[selected].title;
                app.confirmation_popup = Some(ConfirmationPopup {
                    message: format!("Delete “{title}”?"),
                    choice: ConfirmationChoice::Cancel,
                    kind: ConfirmationKind::DeleteFeed,
                });
            }
        }
        KeyCode::Char('t' | 'T') => request_tts(app)?,
        KeyCode::Esc | KeyCode::Left => go_back(app),
        KeyCode::Down | KeyCode::Char('j') => move_selection(app, Direction::Forward),
        KeyCode::Up | KeyCode::Char('k') => move_selection(app, Direction::Backward),
        KeyCode::PageDown if app.screen == Screen::Article => {
            app.scroll_offset = app.scroll_offset.saturating_add(10).min(app.max_scroll);
        }
        KeyCode::PageUp if app.screen == Screen::Article => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }
        KeyCode::Home => select_edge(app, Edge::First),
        KeyCode::End => select_edge(app, Edge::Last),
        KeyCode::Enter | KeyCode::Right => open_selection(app).await?,
        _ => {}
    }
    Ok(())
}

fn request_tts(app: &mut App) -> AppResult<()> {
    if app.tts_downloading {
        app.show_info("Download already in progress.");
        return Ok(());
    }

    if app.tts.is_some() {
        app.show_info("TTS is ready.");
        return Ok(());
    }

    if model_ready() {
        match TTS::load() {
            Ok(tts) => {
                app.tts = Some(tts);
                app.show_info("TTS model loaded.");
            }
            Err(error) => app.show_error(error),
        }
        return Ok(());
    }

    app.confirmation_popup = Some(ConfirmationPopup {
        message: format!(
            "Download Kokoro EN (~330MB) to {}?",
            model_dir().display()
        ),
        choice: ConfirmationChoice::Cancel,
        kind: ConfirmationKind::DownloadTtsModel,
    });
    Ok(())
}

async fn handle_editing_mode(key: KeyEvent, app: &mut App) -> AppResult<()> {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.notice = None;
        }
        KeyCode::Enter => add_feed(app).await?,
        _ => {
            app.input.handle_event(&Event::Key(key));
            app.notice = None;
        }
    }
    Ok(())
}

async fn add_feed(app: &mut App) -> AppResult<()> {
    let url = app.input.value().trim().to_string();
    if url.is_empty() {
        app.show_error("Enter a feed URL.");
        return Ok(());
    }
    if app.feed_list.items.iter().any(|feed| feed.url == url) {
        app.show_error("That feed is already in your list.");
        return Ok(());
    }

    let feed = match load_feed(&url).await {
        Ok(feed) => feed,
        Err(LoadFeedError::Fetch(FetchError::InvalidUrl | FetchError::UnsupportedScheme)) => {
            app.show_error("Enter a valid HTTP or HTTPS feed URL.");
            return Ok(());
        }
        Err(LoadFeedError::Fetch(_)) => {
            app.show_error("The feed request failed. Check the URL and your connection.");
            return Ok(());
        }
        Err(LoadFeedError::Parse) => {
            app.show_error("The URL did not return valid RSS or Atom XML.");
            return Ok(());
        }
        Err(LoadFeedError::NoEntries) => {
            app.show_error("The feed did not contain any readable entries.");
            return Ok(());
        }
    };

    let title = if feed.title.is_empty() {
        "Untitled feed"
    } else {
        &feed.title
    };
    app.feed_list
        .items
        .add_feed(title.to_string(), url.to_string());
    app.feed_list.items.persist()?;
    app.feed_list.state.select_last();
    app.input.reset();
    app.input_mode = InputMode::Normal;
    app.show_info(format!("Added {title}."));
    Ok(())
}

async fn open_selection(app: &mut App) -> AppResult<()> {
    match app.screen {
        Screen::Home => {
            let Some(selected) = app.feed_list.state.selected() else {
                return Ok(());
            };
            let feed = app.feed_list.items[selected].clone();
            let parsed = match load_feed(&feed.url).await {
                Ok(parsed) => parsed,
                Err(LoadFeedError::Fetch(_)) => {
                    app.show_error(format!("Could not load {}.", feed.title));
                    return Ok(());
                }
                Err(LoadFeedError::Parse) => {
                    app.show_error(format!("Could not parse {} as RSS or Atom.", feed.title));
                    return Ok(());
                }
                Err(LoadFeedError::NoEntries) => {
                    app.show_error(format!("{} contains no readable entries.", feed.title));
                    return Ok(());
                }
            };
            app.entry_list.items = parsed.entries;
            app.entry_list.state.select_first();
            app.notice = None;
            app.screen = Screen::Feed;
        }
        Screen::Feed => {
            if let Some(selected) = app.entry_list.state.selected() {
                app.current_entry = app.entry_list.items[selected].clone();
                app.scroll_offset = 0;
                app.max_scroll = 0;
                app.screen = Screen::Article;
            }
        }
        Screen::Article => {}
    }
    Ok(())
}

#[derive(Debug)]
enum LoadFeedError {
    Fetch(FetchError),
    Parse,
    NoEntries,
}

async fn load_feed(url: &str) -> Result<FeedDocument, LoadFeedError> {
    let content = fetch_content(url).await.map_err(LoadFeedError::Fetch)?;
    let feed = parse_feed(&content).map_err(|_| LoadFeedError::Parse)?;
    if feed.entries.is_empty() {
        return Err(LoadFeedError::NoEntries);
    }
    Ok(feed)
}

fn delete_selected_feed(app: &mut App) -> AppResult<()> {
    let Some(selected) = app.feed_list.state.selected() else {
        return Ok(());
    };
    let removed = app.feed_list.items.remove(selected);
    app.feed_list.items.persist()?;

    if app.feed_list.items.is_empty() {
        app.feed_list.state.select(None);
    } else {
        app.feed_list
            .state
            .select(Some(selected.min(app.feed_list.items.len() - 1)));
    }
    app.show_info(format!("Deleted {}.", removed.title));
    Ok(())
}

fn go_back(app: &mut App) {
    app.notice = None;
    match app.screen {
        Screen::Home => {}
        Screen::Feed => app.screen = Screen::Home,
        Screen::Article => {
            app.scroll_offset = 0;
            app.screen = Screen::Feed;
        }
    }
}

fn move_selection(app: &mut App, direction: Direction) {
    match (app.screen, direction) {
        (Screen::Home, Direction::Forward) => app.feed_list.state.select_next(),
        (Screen::Home, Direction::Backward) => app.feed_list.state.select_previous(),
        (Screen::Feed, Direction::Forward) => app.entry_list.state.select_next(),
        (Screen::Feed, Direction::Backward) => app.entry_list.state.select_previous(),
        (Screen::Article, Direction::Forward) => {
            app.scroll_offset = app.scroll_offset.saturating_add(1).min(app.max_scroll);
        }
        (Screen::Article, Direction::Backward) => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
        }
    }
}

fn select_edge(app: &mut App, edge: Edge) {
    match (app.screen, edge) {
        (Screen::Home, Edge::First) => app.feed_list.state.select_first(),
        (Screen::Home, Edge::Last) => app.feed_list.state.select_last(),
        (Screen::Feed, Edge::First) => app.entry_list.state.select_first(),
        (Screen::Feed, Edge::Last) => app.entry_list.state.select_last(),
        (Screen::Article, Edge::First) => app.scroll_offset = 0,
        (Screen::Article, Edge::Last) => app.scroll_offset = app.max_scroll,
    }
}
