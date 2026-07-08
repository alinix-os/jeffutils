# texit

texit — Native text editor for JeffNix / Alinix Distro Usage: texit [path/to/file] A minimal terminal-based text editor similar to nano, with arrow key navigation, basic editing, Ctrl+S to save, Ctrl+X to quit, and Ctrl+W to search.

Este é um utilitário escrito em Rust para o sistema operacional **JeffNix**.

## Uso

```bash
Jefferson S. Rios, texit 1.0 | 2026
Usage: texit [path/to/file]
A terminal-based text editor inspired by nano.
Keys:
  Ctrl+S    Save file
  Ctrl+X    Quit
  Ctrl+W    Search
  Arrows    Navigate
  Home/End  Beginning/End of line
  PgUp/Dn   Page up/down
```

## Compilação

Para compilar este utilitário individualmente:

```bash
cargo build --release
```
