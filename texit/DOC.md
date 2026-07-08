# texit — Editor de Texto Nativo

> **texit** é o editor de texto nativo do JeffNix, parte do JeffUtils.
> Projetado para ser um editor funcional no terminal, similar ao nano,
> porém multiplataforma e integrado ao ecossistema JeffNix.

---

## Sumário

- [Visão Geral](#visão-geral)
- [Arquitetura](#arquitetura)
- [Módulos](#módulos)
  - [1. Terminal (`term.rs`)](#1-terminal-termrs)
  - [2. Buffer (`buffer.rs`)](#2-buffer-bufferrs)
  - [3. Input (`input.rs`)](#3-input-inputrs)
  - [4. Render (`render.rs`)](#4-render-renderrs)
  - [5. File I/O (`fileio.rs`)](#5-file-io-fileiors)
- [Fluxo Principal](#fluxo-principal)
- [Dependências vs Zero Dependências](#dependências-vs-zero-dependências)
- [Exemplo de Implementação (Rascunho)](#exemplo-de-implementação-rascunho)
- [Referências](#referências)

---

## Visão Geral

O `texit` é um editor de texto que opera diretamente no terminal em modo
**raw** (ou **canonical desligado**). Isso significa que cada tecla pressionada
é recebida imediatamente pelo programa, sem buffer de linha e sem echo.

Diferente de editores como `vim` ou `emacs`, o `texit` segue a filosofia **modal
simples** do nano: você começa editando e as teclas de controle (Ctrl+Q, Ctrl+S,
etc.) acionam comandos.

### Funcionalidades Mínimas (MVP)

- Abrir arquivo: `texit caminho/arquivo.txt`
- Criar arquivo novo se não existir
- Navegação com setas, Home, End, PageUp, PageDown
- Inserir e deletar texto (Backspace, Delete, Enter)
- Salvar com Ctrl+S
- Sair com Ctrl+Q
- Status bar na última linha com atalhos e mensagens
- Suporte a scroll horizontal e vertical

---

## Arquitetura

```
src/
├── main.rs        # Ponto de entrada, CLI args, loop principal
├── term.rs        # Controle de terminal (raw mode, tamanho, cores)
├── buffer.rs      # Buffer de texto (linhas, cursor, operações)
├── input.rs       # Leitura e interpretação de teclas
├── render.rs      # Desenho na tela (linhas, status bar, cursor)
└── fileio.rs      # Leitura e escrita de arquivos
```

### Fluxo do Programa

```
main.rs
  │
  ├── 1. Parsear argumentos (caminho do arquivo)
  ├── 2. fileio::ler(caminho) → preencher buffer
  ├── 3. term::entrar_raw_mode()
  ├── 4. Loop principal:
  │      ├── render::desenhar_tela(&buffer)
  │      ├── input::ler_tecla() → Key
  │      ├── buffer::processar(Key)
  │      ├── Se Ctrl+S: fileio::salvar(&buffer)
  │      ├── Se Ctrl+Q: break
  │      └── Se redimensionar: term::obter_tamanho()
  ├── 5. term::sair_raw_mode()
  └── 6. Exibir mensagem final (opcional)
```

---

## Módulos

### 1. Terminal (`term.rs`)

Responsável por:
- Alternar entre **raw mode** e **cooked mode**
- Obter tamanho atual do terminal (linhas e colunas)
- Enviar sequências **ANSI** para controle de cursor e cores

#### Raw Mode

No modo normal (cooked), o terminal:
- Aguarda Enter para enviar dados ao programa
- Ecoa automaticamente o que o usuário digita
- Interpreta sinais como Ctrl+C (SIGINT)

No **raw mode**, precisamos desligar:
- `ECHO` — não ecoar teclas
- `ICANON` — modo linha desligado (entrega imediata)
- `ISIG` — não gerar sinais com Ctrl+C, Ctrl+Z
- `IEXTEN` — desliga Ctrl+V, Ctrl+O do terminal
- `OPOST` — sem processamento de saída (\n → \r\n automático)

##### Exemplo com `termios` (Unix)

```rust
use std::io;
use std::os::unix::io::AsRawFd;

fn entrar_raw_mode() -> io::Result<Termios> {
    let fd = io::stdin().as_raw_fd();
    let mut termios = Termios::from_fd(fd)?;
    let original = termios;

    termios.c_iflag &= !(IXON | ICRNL | INLCR | IGNBRK | BRKINT);
    termios.c_oflag &= !OPOST;
    termios.c_cflag |= CS8;
    termios.c_lflag &= !(ECHO | ICANON | ISIG | IEXTEN);
    termios.c_cc[VMIN] = 0;  // sem mínimo de bytes
    termios.c_cc[VTIME] = 1; // timeout de 100ms

    tcsetattr(fd, TCSAFLUSH, &termios)?;
    Ok(original)
}
```

##### Alternativa com `crossterm` (multiplataforma)

```rust
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

enable_raw_mode()?; // entra em raw mode
// ... edição ...
disable_raw_mode()?; // restaura terminal
```

##### Obter Tamanho do Terminal

```rust
use crossterm::terminal::size;
let (cols, rows) = size()?; // (u16, u16)
```

Ou via ANSI escape: `\x1b[999;999H\x1b[6n` e ler resposta `\x1b[%d;%dR`.

#### ANSI Escapes Essenciais

| Sequência          | Efeito                       |
| ------------------ | ---------------------------- |
| `\x1b[2J`          | Limpa tela inteira           |
| `\x1b[H`           | Move cursor para (1,1)       |
| `\x1b[{y};{x}H`    | Move cursor para linha y, col x |
| `\x1b[?25l`        | Esconde cursor               |
| `\x1b[?25h`        | Mostra cursor                |
| `\x1b[0m`          | Reset de estilos             |
| `\x1b[7m`          | Fundo branco / texto preto   |
| `\x1b[K`           | Limpa até o fim da linha     |
| `\x1b[{n}A`        | Sobe n linhas                |
| `\x1b[{n}B`        | Desce n linhas               |
| `\x1b[{n}C`        | Avança n colunas             |
| `\x1b[{n}D`        | Volta n colunas              |

---

### 2. Buffer (`buffer.rs`)

Gerencia o texto e o estado do cursor.

```rust
pub struct Buffer {
    pub lines: Vec<String>,          // cada linha do texto
    pub cx: usize,                   // coluna do cursor (absoluta)
    pub cy: usize,                   // linha do cursor (absoluta)
    pub scroll_x: usize,            // scroll horizontal
    pub scroll_y: usize,            // scroll vertical
    pub filepath: Option<PathBuf>,  // caminho do arquivo
    pub modified: bool,             // modificado desde último save?
}
```

#### Operações

| Operação            | Descrição                                    |
| ------------------- | -------------------------------------------- |
| `new()`             | Buffer vazio, cursor em (0,0)                |
| `insert_char(c)`    | Insere caractere na posição do cursor        |
| `delete_char()`     | Deleta caractere antes do cursor (backspace) |
| `delete_fwd()`      | Deleta caractere na posição do cursor (del)  |
| `insert_newline()`  | Quebra linha no cursor                       |
| `move_up/down()`    | Move cursor verticalmente                    |
| `move_left/right()` | Move cursor horizontalmente                  |
| `home/end()`        | Início/fim da linha                          |
| `page_up/down()`    | Move página inteira                          |

#### Detalhe: Inserir Caractere

```rust
pub fn insert_char(&mut self, c: char) {
    let line = &mut self.lines[self.cy];
    line.insert(self.cx, c);
    self.cx += 1;
    self.modified = true;
}
```

#### Detalhe: Quebrar Linha (Enter)

```rust
pub fn insert_newline(&mut self) {
    let line = &mut self.lines[self.cy];
    let resto = line.split_off(self.cx);
    self.lines.insert(self.cy + 1, resto);
    self.cy += 1;
    self.cx = 0;
    self.modified = true;
}
```

#### Detalhe: Backspace

```rust
pub fn delete_char(&mut self) {
    if self.cx > 0 {
        self.cx -= 1;
        self.lines[self.cy].remove(self.cx);
        self.modified = true;
    } else if self.cy > 0 {
        let linha_atual = self.lines.remove(self.cy);
        self.cy -= 1;
        self.cx = self.lines[self.cy].len();
        self.lines[self.cy].push_str(&linha_atual);
        self.modified = true;
    }
}
```

---

### 3. Input (`input.rs`)

Leitura de teclas no raw mode.

```rust
pub enum Key {
    Char(char),
    Ctrl(char),         // Ctrl+A até Ctrl+Z
    Enter,
    Tab,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Up, Down, Left, Right,
    Esc,
    Resize,             // terminal redimensionado
    Unknown,
}
```

#### Leitura no Raw Mode (Unix)

Ler byte a byte de `stdin` e interpretar sequências de escape.

```rust
pub fn ler_tecla() -> io::Result<Key> {
    let mut buf = [0u8; 1];
    loop {
        if stdin().read(&mut buf)? == 0 {
            continue; // timeout sem tecla
        }
        return Ok(match buf[0] {
            b'\n' => Key::Enter,
            b'\x7f' => Key::Backspace,
            b'\x1b' => ler_sequencia_escape()?, // ESC →
            b if b < 32 => Key::Ctrl((b + 96) as char),
            c => Key::Char(c as char),
        });
    }
}
```

#### Interpretar Sequências de Escape (setas etc.)

```rust
fn ler_sequencia_escape() -> io::Result<Key> {
    let mut buf = [0u8; 1];
    if stdin().read(&mut buf)? == 0 {
        return Ok(Key::Esc);
    }
    match buf[0] {
        b'[' => {
            let mut seq = vec![];
            loop {
                let mut b = [0u8; 1];
                if stdin().read(&mut b)? == 0 { break; }
                seq.push(b[0]);
                if b[0].is_ascii_alphabetic() || b[0] == b'~' { break; }
            }
            match seq.as_slice() {
                [b'A'] => Ok(Key::Up),
                [b'B'] => Ok(Key::Down),
                [b'C'] => Ok(Key::Right),
                [b'D'] => Ok(Key::Left),
                [b'H'] => Ok(Key::Home),
                [b'F'] => Ok(Key::End),
                [b'2', b'~'] => Ok(Key::Insert),
                [b'3', b'~'] => Ok(Key::Delete),
                [b'5', b'~'] => Ok(Key::PageUp),
                [b'6', b'~'] => Ok(Key::PageDown),
                _ => Ok(Key::Unknown),
            }
        }
        b'O' => {
            // sequências como ESC O H (Home), ESC O F (End)
            let mut b = [0u8; 1];
            stdin().read(&mut b)?;
            match b[0] {
                b'H' => Ok(Key::Home),
                b'F' => Ok(Key::End),
                _ => Ok(Key::Unknown),
            }
        }
        _ => Ok(Key::Unknown),
    }
}
```

---

### 4. Render (`render.rs`)

Desenha o conteúdo do buffer na tela.

#### Estrutura

```rust
pub struct Render {
    pub term_cols: usize,   // largura do terminal
    pub term_rows: usize,   // altura do terminal (incluindo status bar)
}
```

#### Desenhar Tela

A cada frame o renderer:

1. Move cursor para (1,1)
2. Para cada linha visível (scroll_y até scroll_y + term_rows - 2):
   - Escreve a linha com scroll horizontal (scroll_x)
   - Preenche resto da linha com espaços (limpa)
3. Desenha a **status bar** na última linha
4. Posiciona cursor na posição correta

```rust
pub fn desenhar_tela(&self, buffer: &Buffer) {
    let mut out = String::new();
    out.push_str("\x1b[?25l");  // esconde cursor
    out.push_str("\x1b[H");     // cursor para (1,1)

    let linhas_tela = self.term_rows - 1; // 1 linha para status bar

    for y in 0..linhas_tela {
        let idx = buffer.scroll_y + y;
        if idx < buffer.lines.len() {
            let linha = &buffer.lines[idx];
            let visivel = if linha.len() > buffer.scroll_x {
                &linha[buffer.scroll_x..]
            } else {
                ""
            };
            // Limita ao tamanho da tela
            let fim = std::cmp::min(visivel.len(), self.term_cols);
            out.push_str(&visivel[..fim]);
        }
        // Limpa resto da linha
        out.push_str("\x1b[K");
        if y + 1 < linhas_tela {
            out.push_str("\r\n");
        }
    }

    // Status bar
    let status = self.montar_status(buffer);
    out.push_str("\r\n");
    out.push_str("\x1b[7m"); // reverse video
    out.push_str(&status[..std::cmp::min(status.len(), self.term_cols)]);
    out.push_str("\x1b[0m");
    out.push_str("\x1b[K");

    // Posiciona cursor
    let cursor_x = buffer.cx.saturating_sub(buffer.scroll_x) + 1;
    let cursor_y = buffer.cy.saturating_sub(buffer.scroll_y) + 1;
    out.push_str(&format!("\x1b[{};{}H", cursor_y, cursor_x));
    out.push_str("\x1b[?25h"); // mostra cursor

    print!("{out}");
    io::stdout().flush().ok();
}
```

#### Status Bar

```
[texit]  arquivo.txt  Linha 42/100  Col 15   [Modificado]   ^S Salvar  ^Q Sair
```

Pode incluir:
- Nome do programa
- Nome do arquivo
- Posição do cursor (linha/coluna)
- Indicador de modificação
- Atalhos disponíveis

#### Scroll

O scroll deve ser ajustado **antes de desenhar**:

- Se `cy < scroll_y`: `scroll_y = cy`
- Se `cy >= scroll_y + linhas_tela`: `scroll_y = cy - linhas_tela + 1`
- Mesma lógica para `cx` e `scroll_x`

```rust
buffer.ensure_cursor_visible(self.term_cols, self.term_rows - 1);
```

---

### 5. File I/O (`fileio.rs`)

Leitura e escrita de arquivos.

```rust
pub fn ler(caminho: &Path) -> io::Result<Vec<String>> {
    let conteudo = std::fs::read_to_string(caminho)?;
    let linhas: Vec<String> = conteudo
        .lines()
        .map(|l| l.to_string())
        .collect();
    Ok(linhas)
}

pub fn salvar(caminho: &Path, lines: &[String]) -> io::Result<()> {
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        out.push_str(line);
        if i + 1 < lines.len() {
            out.push('\n');
        }
    }
    std::fs::write(caminho, out)?;
    Ok(())
}
```

Observações:
- Arquivo novo (sem caminho) pede nome ao salvar
- Salvar deve criar backup? (depende do design)
- Tratar erros: permissão, disco cheio, path inválido

---

## Fluxo Principal

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // 1. Carregar arquivo ou buffer vazio
    let mut buffer = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        match fileio::ler(&path) {
            Ok(linhas) => Buffer::with_lines(linhas, Some(path)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Buffer::new(Some(path))
            }
            Err(e) => { eprintln!("Erro: {e}"); process::exit(1); }
        }
    } else {
        Buffer::new(None)
    };

    // 2. Entrar em raw mode
    let term = Terminal::new()?;
    term.entrar_raw_mode()?;

    // 3. Loop principal
    let mut render = Render::new(term.cols(), term.rows());
    loop {
        render.desenhar_tela(&buffer);
        let key = input::ler_tecla()?;

        match key {
            Key::Ctrl('s') => {
                let path = buffer.filepath.as_ref()
                    .or_else(|| { /* pedir nome ao usuário */ });
                if let Some(p) = path {
                    fileio::salvar(p, &buffer.lines)?;
                    buffer.modified = false;
                }
            }
            Key::Ctrl('q') => {
                if buffer.modified {
                    // perguntar se quer descartar
                }
                break;
            }
            Key::Char(c)   => buffer.insert_char(c),
            Key::Enter     => buffer.insert_newline(),
            Key::Backspace => buffer.delete_char(),
            Key::Delete    => buffer.delete_fwd(),
            Key::Up        => buffer.move_up(),
            Key::Down      => buffer.move_down(),
            Key::Left      => buffer.move_left(),
            Key::Right     => buffer.move_right(),
            Key::Home      => buffer.go_home(),
            Key::End       => buffer.go_end(),
            Key::PageUp    => buffer.page_up(render.term_rows - 1),
            Key::PageDown  => buffer.page_down(render.term_rows - 1),
            Key::Resize    => render.atualizar_tamanho()?,
            _ => {}
        }
        buffer.ensure_cursor_in_bounds();
    }

    // 4. Sair do raw mode
    term.sair_raw_mode()?;
    Ok(())
}
```

---

## Dependências vs Zero Dependências

O JeffUtils tradicionalmente usa **zero dependências externas**, mas um editor
de texto no terminal é um caso especial. Eis o dilema:

### Abordagem Zero Dependências (JeffUtils padrão)

- Usar `termios` diretamente com `libc` (ou raw `ioctl`)
- Implementar raw mode, tamanho do terminal e ANSI tudo na mão
- **Limitação:** `termios` e ioctl são específicos de Unix. No Windows
  seria necessário usar a API Win32 (`SetConsoleMode`, etc.) via
  `std::os::windows` ou `#[cfg(windows)]`.

Isso quebra o princípio **multiplataforma** se feito só com `libc`.

**Veredito:** Possível, mas exigiria `#[cfg(unix)]` e `#[cfg(windows)]` com
implementações separadas para cada plataforma.

### Abordagem com Dependência (`crossterm`)

[crossterm](https://crates.io/crates/crossterm) é uma biblioteca puramente Rust
que abstrai raw mode, cores, cursor e tamanho do terminal para **Windows, macOS
e Linux** com a mesma API.

```toml
[dependencies]
crossterm = "0.28"
```

```rust
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, size};
use crossterm::cursor::{Show, Hide, MoveTo};
use crossterm::execute;
```

**Vantagem:** Código único, limpo e verdadeiramente multiplataforma.

### Recomendação para JeffUtils

O `texit` é um editor de texto — naturalmente mais complexo que `ls` ou `clock`.
Adicionar `crossterm` como dependência externa é justificável, ou então aceitar
a responsabilidade de manter duas implementações (Unix e Windows) com `#[cfg]`.

Sugestão: **usar `crossterm`** para o MVP e, se no futuro a filosofia "zero
dependências" for mandatória, substituir por implementação própria.

---

## Exemplo de Implementação (Rascunho)

### Cargo.toml

```toml
[package]
name = "texit"
version = "0.1.0"
edition = "2024"

[dependencies]
crossterm = "0.28"
```

### main.rs (estrutura inicial)

```rust
use std::io::{stdout, Write};
use std::path::PathBuf;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, size};
use crossterm::cursor::{Show, Hide, MoveTo};
use crossterm::execute;

mod buffer;
mod input;
mod render;
mod fileio;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Carregar buffer
    let mut buffer = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        match fileio::ler(&path) {
            Ok(linhas) => buffer::Buffer::with_lines(linhas, Some(path)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                buffer::Buffer::new(Some(path))
            }
            Err(e) => {
                eprintln!("texit: erro ao abrir '{}': {}", args[1], e);
                std::process::exit(1);
            }
        }
    } else {
        buffer::Buffer::new(None)
    };

    // Raw mode
    enable_raw_mode().expect("falha ao entrar em raw mode");
    let mut stdout = stdout();
    execute!(stdout, Hide).ok();

    let (term_cols, term_rows) = size().unwrap_or((80, 24));
    let mut render = render::Render::new(term_cols as usize, term_rows as usize);

    loop {
        render.desenhar_tela(&mut stdout, &buffer);

        let key = input::ler_tecla();

        match key {
            input::Key::Ctrl('s') => {
                if let Some(path) = &buffer.filepath {
                    if fileio::salvar(path, &buffer.lines).is_ok() {
                        buffer.modified = false;
                    }
                }
            }
            input::Key::Ctrl('q') => break,
            input::Key::Char(c)   => buffer.insert_char(c),
            input::Key::Enter     => buffer.insert_newline(),
            input::Key::Backspace => buffer.delete_char(),
            input::Key::Delete    => buffer.delete_fwd(),
            input::Key::Up        => buffer.move_up(),
            input::Key::Down      => buffer.move_down(),
            input::Key::Left      => buffer.move_left(),
            input::Key::Right     => buffer.move_right(),
            input::Key::Home      => buffer.go_home(),
            input::Key::End       => buffer.go_end(),
            input::Key::PageUp    => buffer.page_up(render.term_rows - 1),
            input::Key::PageDown  => buffer.page_down(render.term_rows - 1),
            _ => {}
        }
        buffer.ensure_cursor_in_bounds();

        // Detectar redimensionamento
        let (new_cols, new_rows) = size().unwrap_or((80, 24));
        if new_cols as usize != render.term_cols || new_rows as usize != render.term_rows {
            render.term_cols = new_cols as usize;
            render.term_rows = new_rows as usize;
        }
    }

    // Sair do raw mode
    execute!(stdout, Show).ok();
    stdout.flush().ok();
    disable_raw_mode().expect("falha ao sair de raw mode");
    println!("Arquivo salvo com sucesso!");
}
```

---

## Referências

### ANSI Escape Codes
- <https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797>
- <https://en.wikipedia.org/wiki/ANSI_escape_code>

### Crossterm
- <https://crates.io/crates/crossterm>
- <https://docs.rs/crossterm/latest/crossterm/>

### Tutorial de Editor de Texto em Rust
- <https://viewsourcecode.org/snaptoken/kilo/> (inspiração, em C)
- <https://www.philippflenker.com/hecto/> (tutorial Rust, uso de termion)

### Raw Mode / termios
- <https://man7.org/linux/man-pages/man3/termios.3.html>
- <https://docs.rs/nix/latest/nix/termios/index.html>

---

> Este documento é um guia de implementação para o `texit`.
> O código final deve seguir as convenções do JeffUtils:
> `--help`, `--version`, mensagens padronizadas, saída com código de erro.
