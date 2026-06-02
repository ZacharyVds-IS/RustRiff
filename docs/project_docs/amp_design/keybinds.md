# Keybinds

To enhance our user experience, we integrated keybinds. These allow users to quickly access features and perform actions
without navigating through menus or using their mouse.

Our application uses the react "react-hotkeys-hook" package to easily map these binds.

## Default keybinds

RustRiff has the following keybinds:

| Keybind                          | Action                                                  |
|----------------------------------|---------------------------------------------------------|
| number keys                      | Select amp/effects based on index.                      |
| spacebar                         | Toggle the current effect/amp.                          |
| left/right arrow                 | Select the previous/next item in the chain, including the amp, and loop around. |
| shift + left/right arrow         | Move the selected effect left or right in the chain.    |
