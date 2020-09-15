# fractal

A fairly simple fractal renderer



## Features

- Preview window
- Multithreaded background PNG renderer
- Move and zoom around in the preview
- Render the currently visible area in the background in high quality
- Click anywhere to get the coordinates
- Fully configurable with hot reloading

## Usage

Just launch the binary. A custom config file in TOML format can optionally be provided as the first argument, otherwise the defaults listed below will be used.

## Configuration

```toml
max-iterations = 512 # Maximum iterations of f to run before deducing a value is part of the Mandelbrot set

[preview]
width = 320 # Logical horizontal pixels in the preview window
height = 200 # Logical vertical pixels in the preview window
move-factor = 0.125 # Factor for movement relative to the currently visible area
zoom-factor = 1.25 # Factor for zooming relative to the currently visible area

# SDL2 keycodes (https://wiki.libsdl.org/SDL_Keycode)
[preview.keys]
up = "W"
left = "A"
down = "S"
right = "D"
zoom-in = "Up"
zoom-out = "Down"
render = "R"

[render]
width = 3840 # Horizontal pixels in the rendered images
height = 2160 # Vertical pixels in the rendered images
directory = "renders" # Relative or absolute path to a directory where renders will be saved, will be created if it doesn't exist

[gradient]
mode = "HSV" # Gradient mode, either RGB or HSV
# Colours in #rrggbb format (no, "colors" won't be recognised)
colours = [
    "#dd2222",
    "#22dd22",
    "#2222dd",
]
# Number of times the gradient cycles
cycles = 1
```
