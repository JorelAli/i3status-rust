# "Widget toolset"
- Icon = spanner.

When you click it, it opens up a list of other widgets (expands to the left).

Widgets include:
- One button dice roll (icon = dice, when you click it the dice icon changes using FontAwesome's dice icons)
- One button is a counter (might be useful?!). When you left click it goes up, when you right click it resets back to 0
- ??

# Workspace manager
Maybe something which lets you create workspaces for certain programs? e.g. a [toml] file consisting of something like:

[WorkspaceName]
programs = ["konsole -e blah", "qutebrowser http://link"]

### Workspace manager example

[Java]
programs = ["eclipse"]

[Rust]
programs = ["konsole -e ~/github/i3status-rust", "qutebrowser http://www.rustlang.whatever"]
icon = "whatever fontawesome rust icon"
