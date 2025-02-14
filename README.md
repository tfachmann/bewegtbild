# Bewegtbild

A video playable PDF viewer built for presentations.
`bewegtbild` let's your images move.

> [!WARNING]
> Under construction, but usable!

## Usage

```sh
# View a PDF
bewegtbild test.pdf

# View a PDF + video configuration when and where to play videos, GIFs, ...
bewegtbild test.pdf -c ~/foo_config.json

# PDF + video configuration, updates when configuration changes (helps to write config)
bewegtbild test.pdf -c ~/foo_config.json --reload
```

Example configuration

```json
[
  {
    "video_path": "./test_other.gif",
    "slide_num": [3, 7],
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

## Installation

This installation requires a pre-built library.
Download and extract the version [pdfium/6541](https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F6541) of [pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) under `pdfium/linux-x64`.

### Prepare

```sh
git clone https://github.com/tfachmann/bewegtbild/tree/main
cd bewegtbild
mkdir pdfium
cd pdfium
mkdir linux-x64
# download and extract libpdfium.so (or libpdfium.a) here
#   visit https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F6541
#   or build it yourself
```

### Build

```sh
cargo build --release
```

If a static library `libpdfium.a` is provided, use this instead. Note that [pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) does not provided static libraries. To use a `libpdfium.a`, one must build pdfium from source.

```sh
cargo build --release --features static
```

## Vision

- [x] fast, no lags
- [x] full PDF support -- via [pdfium-render](https://github.com/ajrcarey/pdfium-render)
- [x] full video format support (GIF, mp4, mkv, ...) -- via [egui-video](https://github.com/n00kii/egui-video)
- [x] native (linux)
- [ ] web
- [ ] easy installation (!hard because of pdfium)
- [ ] minimal features
  - [x] vim-like navigation
  - [x] presenter support
  - [x] play (loop) videos
  - [x] videos spanning multiple slides
  - [ ] pause videos
  - [ ] configurable hotkeys
- [ ] easy incorporation of videos (no code)
  - [x] via configuration
  - [ ] via python launcher script (supported `dataclass`)
- [ ] built-in support for presentations built with `typst` (polylux?)
