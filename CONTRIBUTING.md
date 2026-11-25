## Contributing guidelines
This file defines certain rules contributors have to follow, and provides basic guidance on how to
work with the project's codebase.


## Use of generative AI
See the [AI Usage Policy](https://github.com/ColdMaterial/iced_md3/blob/main/AI_POLICY.md).

TLDR: Only text and code are allowed; you must fully understand what you submit.


## Development commands

Development commands are bundled in the `justfile` in the repo root. This file can be used with the
[`just`](https://github.com/casey/just) program, or, if you don't want to install `just`, pasted
into your terminal.

Mentioned commands can be run as:
```bash
just <command name>
```

### Generating documentation
There are three commands for generating documentation:
- `doc-no-all-features` generates documentation with all optional features disabled
- `doc` generates documentation with all features enabled
- `open-doc` generates documentation with all features enabled, and opens it in the default browser

> [!NOTE]
> Generating documentation requires the nightly toolchain of Rust to be installed.

### Formatting
Follow standard Rust formatting. Format with `fmt`, check with `fmt-check`:
```bash
just fmt
just fmt-check
```

### Linting
The project is linted with [Clippy](https://doc.rust-lang.org/clippy/) locally and in CI, where
warnings are treated as errors, so fix warnings instead of ignoring them. Check with the `clippy`
command:
```bash
just clippy
```
Clippy configuration lives in the `[lints]` table in `Cargo.toml`. Inline `#[allow]` attributes are
kept to a minimum, and when one is truly needed, it carries a comment explaining why.

### Testing
The project uses unit tests and [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) for CI.

#### Compilation/Unit tests
Check whether the project compiles and if all tests pass with the `test` command. It will compile
the project twice - once with all optional features enabled, and once with all features disabled.
This is to ensure that everything compiles and works for all configurations.

#### Cargo deny
[Cargo deny](https://github.com/EmbarkStudios/cargo-deny) checks for incompatible licenses,
[advisories](https://rustsec.org/advisories/), etc. Check with `deny`.


### All checks
You can also run all the mentioned checks with the `check` command.


## Architecture
The following is a tree of certain subdirectories in the `src/` directory, with brief explanations
of what each one holds. Refer to the documentation inside those modules for more info.

```
|- animation        Animation utils, easing curves, motion physics
|- spacing.rs       Material Design 3 spacing tokens
|- style            Color-mixing utils, state layer, shadows
|- theme            Implementation of the Material color system used for styling library widgets
|- widget           Implementations of the Material widgets
    |- advanced     Additional utility widgets that can be combined with the material widgets or standard widgets from Iced
    |- internal     Common internal functionality not to be exposed publicly
    |- ripple       Implementation of the Material ripple effect, used by various widgets in the library
    |- wrappers     Small wrapper widgets that can easily be defined with a single function
```

## Conventions
Conventions for styling, naming, and ordering of parameters, among other things. You should
generally stick to the conventions in Iced, unless stated otherwise here.

### Styling
Iced has certain limitations in terms of styling, so styling is done by manually passing a
`ColorScheme` implementor to widget constructors. Widgets never store that implementor, but rather
construct a custom `Style` struct with public fields containing all the design tokens needed for the
widget.

- Widgets must **never** hardcode color tokens.
- `Style` structs must provide one or more ways to be constructed with a `(ColorScheme + ?Sized)` implementor reference.
- Widget constructors should take a `(ColorScheme + ?Sized)` implementor reference, and construct the style from it.
- Widgets must provide constructor(s) where a custom `Style` is passed, instead of a theme.
- Widgets must never store a `ColorScheme` dyn object, or a concrete `ColorScheme` implementor.

### Code organization
- Non-material utility widgets go to `advanced`.


## Notes on upstream blockers
Certain features of Iced MD3 are missing because the functionality required to implement them is not
yet present in upstream Iced. Do not attempt to invent hacky workarounds for those issues when
contributing to the project, but rather try to resolve the missing functionality in upstream first.

Some of the missing Iced features blocking parts of the development of Iced MD3:

### Layer opacity in `draw`
Iced has no way to apply an opacity value to a widget being drawn, which blocks all sorts of opacity
animations for widgets where the children being drawn are not provided by the library itself. Eg.
the opacity of the dialog content can't be enforced, because its content is ultimately
user-controlled.

### Radial gradients
The implementation of the Material ripple is one example of a hacky solution to an Iced limitation.
It draws several (31) rings from the ripple origin to approximate a radial gradient.

### Rounded rect clipping for arbitrary widgets
There's currently no straightforward way of clipping widget draws against a rounded rectangle. This
would be useful primarily for the ripple, which as of now is clipped with a polygon.

### Advanced shadows
Iced has a very simple way of drawing shadows - just a blurred silhouette of the widget with an
offset. MD3, on the other hand, has layered shadows for its elevation systems, which can't be
replicated with the current shadow implementation on Iced. This library approximates the material
elevation system with Iced shadows.
