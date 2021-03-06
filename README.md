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

To use the default configuration listed below, just start the program.

A custom configuration can be provided to override the defaults by providing the config file as the first argument to the program (this can be done by just drag-and-dropping the file onto the program).

It is not necessary to provide all options, the defaults will be used for any missing values.

The program supports config hot reloading, which means changes to the config file will be applied in real time when they are saved to disk.

## Configuration

```toml
max-iterations = 512 # Maximum iterations of f to run before deducing a value is part of the Mandelbrot set
black = "#000000" # Colour of the points part of the set in #rrggbb format

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
