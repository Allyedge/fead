# Fead

A small terminal reader for RSS and Atom feeds.

## Run it

```sh
cargo run
```

Fead stores subscriptions in `feeds.json` in the directory where you run it.

## Keys

| Key                     | Action                            |
| ----------------------- | --------------------------------- |
| `↑` / `↓` or `j` / `k`  | Move or scroll                    |
| `Enter` / `→`           | Open the selected feed or article |
| `Esc` / `←`             | Go back                           |
| `a` or `/`              | Add a feed from the home screen   |
| `Backspace` / `Delete`  | Delete the selected feed          |
| `Page Up` / `Page Down` | Scroll an article by a page       |
| `q` or `Ctrl-C`         | Quit                              |
