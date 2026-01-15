# Language Specification

The language used to visualize data.

## Example

```bash
canvas map
layer grid
grid occ w=50 h=50 cell=0.2

layer history
draw cell occ x=10 y=10 color=blue

layer entities
shape drone circle r=0.3
shape drone line x1=0 y1=0 x2=0.5 y2=0 color=red

entity uav shape=drone x=0 y=0 angle=0deg
move uav x=5 y=5 angle=45deg
```

## Commands Reference

Angle brackets `<>` denote parameter.
Square brackets `[]` denote optional parameters.

E.g. `command <param1> [param2]`

Types of parameters:

- `float`: Floating point number (e.g. `3.14`, `-0.001`, `2.0`)
  - Default: `0.0`
- `int`: Integer number (e.g. `42`, `-7`, `0`)
  - Default: `0`
- `uint`: Unsigned integer (e.g. `0`, `1`, `42`), i.e. non-negative integers
  - Default: `0`
- `angle`: Angle in degrees or radians (e.g. `90deg`, `1.57rad`)
  - Default: `0deg`
- `color`: Color name or hex code (e.g. `red`, `#FF0000`, `rgb(255,0,0)`)
  - Default: `black`
- other: String or identifier (e.g. `my_shape`, `main_canvas`)

### Frames

Frames are coordinate systems that can be defined relative to other frames.
Default frame is `world`.

#### Create or update a new Frame

Create or update a new reference frame.

```js
frame set <name> x=<float> y=<float> yaw=<angle> [parent=<parent_name>]
```

- `<name>`: Name of the new frame.
- `x`, `y`: Position of the new frame relative to its parent frame.
- `yaw`: Orientation of the new frame relative to its parent frame.
- `parent`: (Optional) Name of the parent frame. Default is `world`.

#### Select Frame

Select the current reference frame.
From this point onward, all position and orientation commands are interpreted relative to the selected frame.

```js
frame select <name>
```

- `<name>`: Name of the frame to select.

### Canvas

Canvas is the main drawing area where all visual elements are rendered.
Creating a new render is like opening a new drawing window.

#### Create or select a Canvas

Create or select a canvas by name.
Default canvas is `main`.

```js
canvas <name>
```

- `<name>`: Name of the canvas to create or select.
- If the canvas with the specified name does not exist, it will be created and selected.
- If it already exists, it will be selected for subsequent drawing commands.

### Layers

Layers are used to organize the vertical order of visual elements within a canvas.

#### Create or select a Layer

Create or select a layer by name within the *currently selected* canvas.
Default layer is `default`.

```js
layer <name>
```

- `<name>`: Name of the layer to create or select.
- If the layer with the specified name does not exist within the current canvas, it will be created and selected.
- If it already exists, it will be selected for subsequent drawing commands.

### Shapes

Shapes are reusable geometric objects that can be drawn multiple times in the scene.

#### Create a Shape

Create a new shape with the specified name.
Keep adding primitives to the shape for defining its geometry.

Does not draw the shape yet, just defines it for later use.

```js
shape <name> <primitive> <...parameters>
```

Example:

```js
shape <name> line x1=<float> y1=<float> x2=<float> y2=<float> [color=<color>] [thickness=<float>]
shape <name> circle x=<float> y=<float> r=<float> [color=<color>] [thickness=<float>]
```

- `<name>`: Name of the shape to create or modify.
- `<primitive>`: A shape primitive (e.g. line, circle, polygon) that defines the geometry of the shape.

You can add multiple primitives to the same shape by repeating the `shape <name> <primitive>` command.

### Entity

Entities are movable objects in the scene that can be referenced and manipulated.

#### Create an Entity

Create a new entity with the specified name using a defined shape.

```js
entity create <entity_name> shape=<shape_name> x=<float> y=<float> [angle=<angle>]
```

- `<entity_name>`: Name of the new entity.
- `<shape_name>`: Name of the shape to use for the entity.
- `x`, `y`: Initial position of the entity in the current frame.
- `angle`: (Optional) Initial orientation of the entity. Default is `0deg`.
- The entity is drawn at the specified position and orientation.

#### Move an Entity

Move an existing entity to a new position and orientation.

```js
entity move <entity_name> x=<float> y=<float> [angle=<angle>]
```

- `<entity_name>`: Name of the entity to move.
- `x`, `y`: New position of the entity in the current frame.
- `angle`: (Optional) New orientation of the entity. If not specified, the orientation remains unchanged.
- The entity is updated to the new position and orientation in the scene.

### Drawing Primitives

Drawing primitives are basic geometric shapes that can be drawn directly onto the current layer.
These cannot be moved later like entities.

#### Draw a Primitive

Draw a primitive or a shape directly onto the current layer.

```js
draw <primitive> <...parameters>
draw <shape_name>
```

#### Primitive Types

- `line`: Draw a line segment.
  - Parameters: `x1=<float> y1=<float> x2=<float> y2=<float> [color=<color>] [thickness=<float>]`
  - Description: Draws a line from point `(x1, y1)` to point `(x2, y2)`.
- `circle`: Draw a circle.
  - Parameters: `x=<float> y=<float> r=<float> [color=<color>] [thickness=<float>]`
  - Description: Draws a circle centered at `(x, y)` with radius `r`.
- `rectangle`: Draw a rectangle.
  - Parameters: `x=<float> y=<float> w=<float> h=<float> [color=<color>] [thickness=<float>]`
  - Description: Draws a rectangle with the top-left corner at `(x, y)`, with specified width and height.
- `polygon`: Draw a polygon.
  - Parameters: `points=[(x1,y1),(x2,y2),...] [color=<color>] [thickness=<float>]`
  - Description: Draws a polygon defined by a list of points.
