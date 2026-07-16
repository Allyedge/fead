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
| `t`                     | Optional TTS model download/load  |
| `q` or `Ctrl-C`         | Quit                              |

## TTS (optional)

Press `t` to download the sherpa-onnx Kokoro English model if you want TTS. It is not bundled. Files go in `models/kokoro-en-v0_19/` next to where you run the app (same idea as `feeds.json`). Delete that folder to remove the model.
