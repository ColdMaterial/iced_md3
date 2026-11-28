<h1 align="center">iced_m3</h1>

<div align="center">
  
  ![](https://img.shields.io/github/last-commit/ColdMaterial/iced_md3?&style=for-the-badge&color=FFFFFF&logo=git&logoColor=C9C9C9&labelColor=252525)
  ![](https://img.shields.io/github/repo-size/ColdMaterial/iced_md3?&style=for-the-badge&color=FFFFFF&logo=git&logoColor=C9C9C9&labelColor=252525)
</div>

<div align="center">
  Implementation of the <a href="https://m3.material.io">Material Design 3</a> system for the <a href="https://iced.rs/">Iced GUI library</a> 
</div>

# Features
- [Motion Physics](https://m3.material.io/styles/motion/overview)
- [Dynamic Color](https://m3.material.io/styles/color/system)
- [Elevation](https://m3.material.io/styles/elevation/overview)
- Support for text or SVG icons

Try the web  [showcase](https://coldmaterial.github.io/iced_m3/)!

You can go see the demos [here](https://github.com/ColdMaterial/iced_md3/blob/main/demos/)!

[Chilen](https://github.com/tpaau/chilen) uses `iced_m3` extensively, you can check how the library
is used there too!


# Usage
Run this in your project:
```bash
cargo add iced_m3 --git https://github.com/ColdMaterial/iced_md3.git
```

This library is largely for use in my personal projects (like
[Chilen](https://github.com/tpaau/chilen)), and constantly changing, so consider pinning a commit if
you wish to use it in your project. I also don't plan on releasing this on
[crates.io](https://crates.io) anytime soon, largely for the same reason.

`iced_m3` is compatible with Iced **`0.14.0`** and will most likely be kept up to date with stable
Iced releases.

# Advanced
Widgets used internally by `iced_m3` for other widgets that may also be useful outside of `iced_m3`.
Those are gated behind the `advanced` feature.
