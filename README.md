# orinfar

a text editor for witches

# Building
To build this project, first install [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), the Rust package manager, then clone this repo and `cargo run` in the project directory.

The editor is inspired by Vi, but has a few key differences, so you should read the [user manual](https://github.com/azaleacolburn/orinfar/tree/main/docs/MANUAL.md) before using it in a meaningful manner.

# Principles

## Minimal

- A TUI text editor
- Simple but powerful and opinionated
- Not configurable
- Similar to base Vi, but with better defaults (lsp support, etc)
- Similar in princible to [Helix](https://helix-editor.com/), but more minimal

## Reliable

- Ideally, all software should work exactly as documented and intented
    - No implementation quirks
    - No undocumented but "obvious" or "intuitive" behavior
- Not much else to say at the moment

# User Manual
Please read the [user manual](https://github.com/azaleacolburn/orinfar/tree/main/docs/MANUAL.md) before trying to edit text with Orinfar. While this text editor is mostly a subset of Vi, there are plenty of differences, both subtle and unsubtle.

# Implementation

# Roadmap

I'll specify and revise this list when I get around to specific things and I have a better understanding of where I want to take this project

- [x] Basic Vi Commands and Features (see the [user manual](https://github.com/azaleacolburn/orinfar/tree/main/docs/MANUAL.md))
- [ ] Additional Vi Features
    - [x] Undo System (not comprehensively documented)
    - [ ] Redo System
    - [x] Status Bar (not comprehensively documented)
    - [x] Minimal Register System (not yet documented)
    - [ ] Robust Register System (idk what that means exactly)
    - [ ] Mark System
    - [ ] Text Objects
- [ ] Advanced Features
    - [ ] Syntax Highlighting (tree-sitter)
    - [ ] LSP Client Support

# Feature/Bug Requests

If I haven't implemented your favorite Vi feature, or you've found some undesirable bug or behavior, feel free to reach out to me at `azaleacolburn[AT]gmail[DOT]com`.


# Known Bugs
Please check the [stable](https://github.com/azaleacolburn/orinfar/tree/stable)  for the latest bug-free commit
