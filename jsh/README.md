# 🐚 jsh — Shell Interativo do Alinix-OS

O `jsh` é o shell interativo padrão e motor de scripting do ecossistema **JeffNix / Alinix-OS**. Desenvolvido em Rust para ser rápido, robusto e moderno, ele combina a compatibilidade de recursos clássicos do POSIX/Bash com a experiência inteligente de shells modernos como Fish, Zsh e Nushell.

---

## ✨ Recursos de Destaque

### 🧠 Sistema de Histórico Inteligente (Redesenhado)
O motor de histórico do `jsh` foi completamente reprojetado para ir além de um simples arquivo de comandos:
* **Banco de Dados JSONL**: O histórico é salvo de forma permanente e assíncrona em `~/.local/share/jsh/history` utilizando o formato JSON Lines, garantindo integridade de dados e proteção contra corrupção.
* **Metadados Ricos**: Cada entrada armazena o comando, carimbo de data/hora (timestamp ISO 8601), diretório de trabalho (`cwd`), código de saída (`exit_code`), contagem de execuções e último uso.
* **Sincronização em Tempo Real**: Múltiplas abas abertas do shell compartilham o mesmo histórico instantaneamente através de leitura incremental via *seek* otimizado.
* **Comandos Favoritos**: Fixe comandos frequentes usando `history pin <comando>` (e `history unpin`). Comandos fixados recebem a maior pontuação em sugestões automáticas.
* **Ignora Duplicados e Filtros**: Filtra comandos vazios, comandos internos como `history`, `clear`, `exit`, duplicados consecutivos e obedece a padrões da variável `$HISTIGNORE`.

### 💡 Sugestões Automáticas Estilo Fish
À medida que você digita, o `jsh` analisa o histórico e apresenta a sugestão mais provável em cinza:
* **Algoritmo de Ranking**: As sugestões são calculadas em **menos de 5ms** usando um sistema de pontuação avançado:
  $$\text{Score} = (\text{Filtro de Prefixo} \times 100) + (\text{Mesmo Diretório} \times 50) + (\text{Frequência} \times 0.6) + (\text{Recência} \times 0.4) + \text{Favorito Pinned}$$
* **Navegação Local-First**: Ao navegar com as setas **Para Cima/Para Baixo**, o shell prioriza comandos executados no diretório atual antes de buscar no histórico global.
* **Aceitação Prática**: Pressione a seta **Para a Direita** ou a tecla **End** para aceitar a sugestão em tempo real.

### 🔍 Busca Reversa Fuzzy Interativa (Ctrl+R)
Substituindo a busca clássica do readline, o `jsh` introduz um finder interativo inline:
* **Pesquisa Fuzzy e Substring**: Digite qualquer parte ou padrão subsequente do comando (ex: `gp` para achar `git push`).
* **Interface Integrada**: Um menu limpo de 5 correspondências aparece instantaneamente abaixo da sua linha atual com um cursor de seleção (`>`).
* **Navegação Segura**: Escolha o comando com as setas direcionais, pressione **Enter** para executar ou **Esc** para cancelar sem quebrar o buffer atual do terminal.

### 🚀 Outras Funcionalidades
* **Prompt Dinâmico e Colorido**: Renderiza o ícone do sistema operacional ativo (🐧/🍎/🪟) e o diretório atual de forma inteligente, sem corromper as coordenadas de cursor.
* **Expansão de Histórico (Bash-style)**: Suporte completo a `!!` (último comando), `!$` (último argumento), `!n` (comando de ID específico), `!prefixo` e `!?subtexto`.
* **Jeofetch Embutido**: O comando `jeofetch` vem integrado e é executado opcionalmente na inicialização do shell em terminais interativos.
* **Sintaxe de Shell Robusta**: Pipes (`|`), redirecionamentos (`>`, `>>`, `<`, `2>`, `&>`, etc.), heredocs (`<<`, `<<<`), listas de comandos (`;`, `&&`, `||`) e execução em segundo plano (`&`).
* **Fallback Inteligente para Bash**: Scripts que usem construções avançadas e funções do Bash (como arquivos `.jshrc` que chamam `nvm` ou `rvm`) são mapeados automaticamente e executados em segundo plano via `bash -ic` transparente.

---

## 🛠️ Compilação

Certifique-se de ter o `cargo` instalado em seu sistema:

```bash
cargo build --release
```

O executável final otimizado será gerado em `target/release/jsh`.

---

## 🎨 Fonte JSH Mono

Para renderizar perfeitamente os logos minimalistas no prompt do `jsh`, o projeto inclui a fonte **JSH Mono** (localizada em `assets/font/`). 

Ela é um fork da aclamada [JetBrains Mono] (https://www.jetbrains.com/lp/mono/) na qual os caracteres 🐧 (`U+1F427`), 🍎 (`U+1F34E`) e 🪟 (`U+1FA9F`) foram fundidos com os logotipos minimalistas do Linux, macOS e Windows.

### Instalação Rápida da Fonte:

```bash
mkdir -p ~/.local/share/fonts
cp assets/font/*.ttf ~/.local/share/fonts/
fc-cache -f ~/.local/share/fonts
```

Depois de instalar, altere a fonte da janela do seu emulador de terminal para **JSH Mono** para ver os ícones integrados perfeitamente ao prompt.

---

## ⚙️ Configuração

O comportamento do histórico e do shell pode ser personalizado criando o arquivo `~/.config/jsh/config.toml`:

```toml
[history]
history = true
history_size = 100000
autosuggestion = true
fuzzy_history = true
share_history = true
history_sync = true
ignore_duplicates = true
```
