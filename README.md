# RSML - Robot Scene Markup Language

RSML (Robot Scene Markup Language) is a 3D markup language that serves as a 3D equivalent of HTML and CSS, designed for creating 3D spatial layouts and scenes. It allows you to define 3D environments using familiar web technologies concepts, making 3D scene creation more accessible and intuitive.

## Overview

RSML combines the structural simplicity of HTML with the styling power of CSS to create 3D scenes. You can define 3D objects, their relationships, and their styling using a familiar markup syntax.

## Features

- **HTML-like structure**: Define 3D scenes with familiar `<div>`, `<object>`, and `<group>` elements
- **CSS-like styling**: Use selectors and properties to style 3D objects with flexbox-inspired layout rules
- **Flexbox-inspired layout**: Apply flexbox concepts in 3D space with properties like `flex-direction`, `justify-content`, and `size`
- **Current output**: Compiles to MJCF (Multi-Joint dynamics with Contact Format) for physics simulation
- **Future support**: Planned support for Three.js for web-based 3D rendering

## Installation

```bash
# Clone the repository
git clone <repository-url>

# Build the project
cargo build --release
```

## Usage

Create an RSML file with the `.rsml` extension (or `.xml` as shown in the example):

### Example

```xml
<rsml>
  <head>
    <style> 
    <![CDATA[
      # 格式是toml
      [[styles]] 
      selector = "#table-legs" # id选择器
      size = "100% 100% auto" # 分别表示x,y,z的长度(对应css的width和height), 100%表示长度为父容器的对应轴的长度， 
                              # auto表示长度根据子容器确定, 因为叶子节点的size必须是确定的, 所以auto是一定能被求解的。

      flex-direction = "y"    # 表示flex的主轴为y轴，并且flex的方向是原点到z轴的正无穷方向(0,1,0)
      justify-content = "space-between" # 和css的语义一致

      [[styles]]
      selector = "#left-legs, #right-legs" 
      flex-direction="x" 
      size="100% auto auto"
      justify-content="space-between"

      [[styles]]
      selector = ".leg" # class选择器
      margin-x = "10cm"
      margin-y = "10cm"
      margin-x-reverse = "10cm"
      margin-y-reverse = "10cm"

      [[styles]]
      selector = "#on_table"
      flex-direction="x"

      [[styles]]
      selector = ".bottles"
      margin-x = "10cm"
      margin-y = "10cm"
      margin-x-reverse = "10cm"
      margin-y-reverse = "10cm"
      ]]>
    </style>
  </head>
  <body>
    <!-- flex默认-z，就是按照顺序从上到下排列-->
    <div id="main_room" style="size:10m 10m 10m;">
      <div id="table_area">
        <div id="on_table">
          <group select="random" class="bottles"> bottles </group>   <!-- 从bottle group中随机-->
          <group select="random" class="bottles"> bottles </group>   <!-- 从bottle group中随机-->
        </div>
        <object id="table_plane"> table_plane </object>
        <div id="table-legs">
          <div id="left-legs">
            <object id="leg1" class="leg"> table_leg </object>
            <object id="leg2" class="leg"> table_leg </object>
          </div>
          <div id="right-legs">
            <object id="leg3" class="leg"> table_leg </object>
            <object id="leg4" class="leg"> table_leg </object>
          </div>
        </div>
      </div>
      <object id="floor"> floor </object>
    </div>
  </body>
</rsml>
```

### Building

Compile your RSML file to MJCF:

```bash
cargo run -- <input.rsml> <output.mjcf>
```

## Elements

- `<rsml>`: Root element of the document
- `<head>`: Contains styling and metadata
- `<style>`: Contains CSS-like styling rules using TOML format
- `<body>`: Contains the 3D scene structure
- `<div>`: Container element for grouping objects in 3D space
- `<object>`: Represents a 3D object in the scene
- `<group>`: Represents a group of objects with selection options

## Style Properties

- `size`: Defines the size along X, Y, Z axes (e.g., "100% 100% auto")
- `flex-direction`: Determines the primary axis for layout ("x", "y", or "z")
- `justify-content`: Distributes space along the main axis ("space-between", etc.)
- `margin-x`, `margin-y`, `margin-x-reverse`, `margin-y-reverse`: Define margins in 3D space

## Output Formats

- **MJCF**: Currently supported output format for physics simulation engines
- **Three.js**: Planned future support for web-based 3D rendering

## Roadmap

- Support for additional 3D output formats
- Enhanced styling capabilities
- Animation support
- Three.js integration

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[License information would go here]