use crate::app::{App, AppResult, InputMode};
use crate::feed::{
    entries::FeedDocument,
    feeds::FeedsManager,
    fetch::{fetch_content, FetchError},
    reader::parse_feed,
};
use crate::screen::Screen;

#[derive(Debug)]
pub(super) enum LoadFeedError {
    Fetch(FetchError),
    Parse,
    NoEntries,
}

pub(super) async fn load_feed(url: &str) -> Result<FeedDocument, LoadFeedError> {
    let content = fetch_content(url).await.map_err(LoadFeedError::Fetch)?;
    let feed = parse_feed(&content).map_err(|_| LoadFeedError::Parse)?;
    if feed.entries.is_empty() {
        return Err(LoadFeedError::NoEntries);
    }
    Ok(feed)
}

pub(super) async fn add_feed(app: &mut App) -> AppResult<()> {
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

pub(super) async fn open_selection(app: &mut App) -> AppResult<()> {
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

pub(super) fn delete_selected_feed(app: &mut App) -> AppResult<()> {
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
