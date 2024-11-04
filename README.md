# Bewegtbild

A video playable PDF viewer built for presentations.
`bewegtbild` let's your images move.

## Usage

```sh
# View a PDF
bewegtbild test.pdf

# View a PDF + video configuration when and where to play videos, GIFs, ...
bewegtbild test.pdf -c ~/foo_config.json
```

Example configuration

```json
[
  {
    "video_path": "./test_other.gif",
    "slide_num": 7,
    "pos": ["70%", "70%"],
    "size": ["20%", "20%"]
  },
  {
    "video_path": "./test.mkv",
    "slide_num": 5,
    "pos": ["40%", "50%"],
    "size": ["40%", "40%"]
  },
  {
    "video_path": "./test.mkv",
    "slide_num": 3,
    "pos": ["0%", "10%"],
    "size": ["100%", "100%"]
  }
]

```

## Vision

- fast, no lags
- full PDF support
- full video format support (GIF, mp4, mkv, ...)
- native and web
- minimal features
- easy incorporation of videos (no code)
  - via configurations
  - via python launcher script (supported `dataclass`)
- built-in support for presentations built with `typst` (polylux?)
