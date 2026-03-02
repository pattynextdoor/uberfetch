<p align="center">

<img src="uberfetchdemo.gif" width="600" alt="uberfetch demo" />

<br/>

<h3>☄️ system info with esoteric default visuals</h3>

[![CI](https://img.shields.io/github/actions/workflow/status/pattynextdoor/uberfetch/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/pattynextdoor/uberfetch/actions/workflows/ci.yml)

</p>

---

I wanted arcane shapes to thrive in the corner of my terminal.

## Install

```sh
cargo install uberfetch
```

Or from source:

```sh
git clone https://github.com/pattynextdoor/uberfetch.git
cd uberfetch
cargo install --path .
```

Run `uberfetch`. Stare.

## Usage

```sh
uberfetch                        # default → rotating diamond
uberfetch --animation toroid     # toroidal particle flow
uberfetch --animation hypercube  # 4D tesseract projection
uberfetch --animation geodesic   # breathing geodesic sphere
uberfetch --fps 60               # smoother on fast terminals
uberfetch --list                 # list all animations
```

Press `q`, `Esc`, or `Ctrl+C` to leave (if you can).

## Animations

| Animation | What you see |
|-----------|-------------|
| **diamond** | Rotating pulsating octahedron with glow trails |
| **hypercube** | 4D tesseract projected into 3D, dual-axis rotation |
| **toroid** | Particles flowing along a torus surface |
| **geodesic** | Icosphere with breathing vertex oscillation |

All rendered as braille-dot wireframes — no GPU, no sixel, just Unicode.

## License

[MIT](LICENSE)
