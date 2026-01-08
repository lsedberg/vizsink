# VizSink

A command-line tool for visualizing data streams in real-time.

## Features

- Real-time data visualization

## How it Works

VizSinks reads lines from standard input and visualizes the data in a browser.

It works by piping data into the VizSink command.

This data follows a simple format where each line represents a command.

### Commands

#### Create a Grid ?

```bash
grid width height [cell_size]
```

#### Draw a Cell

Draws a square relative to a grid's origin, and sizes it according to the grid's cell size.

```bash
cell x y color
```

#### Create a Color

```bash
color name hex

# Example:
color red #ff0000

# This color can now be used in other commands.
cell x y red
# or use the hex code directly
cell x y #ff0000
```
