# solo_chess

Um port do [Solo Chess do chess.com](https://www.chess.com/solo-chess) pra rodar no terminal, escrito em Rust.

Solo Chess é um quebra-cabeça de xadrez pra um jogador só: **todo lance tem que ser uma captura**, cada peça pode capturar no máximo duas vezes, e se tiver um rei ele precisa sobrar por último. Você resolve quando sobra uma peça só. É um jeito enxuto de treinar **look-ahead** — montar a cadeia inteira de capturas na cabeça antes de encostar na primeira peça.

A ideia nasceu de jogar isso no chess.com e querer uma versão sem distração, só teclado, rodando no terminal — e de transformar o exercício num treino de verdade pra quem pratica visualização: enxadristas calculando linhas e speedcubers fazendo look-ahead. O xadrez é o que o jogo treina diretamente (calcular uma sequência forçada antes de executar); pra quem cuba, o que transfere é o hábito de planejar a sequência inteira e segurar ela na memória de trabalho antes de "executar".

## Modo treino

Cada puzzle passa por três fases, pensadas pra forçar o planejamento em vez de tentativa e erro:

1. **Inspeção** — o tabuleiro aparece e o cronômetro de inspeção começa a contar. Você estuda a posição e planeja a cadeia de capturas. Quando estiver pronto, `Enter` começa a resolver.
2. **Resolução** — o cronômetro de execução roda enquanto você joga. Por padrão as casas de captura aparecem como pontinhos; aperte `h` pra desligar as dicas e resolver no escuro (look-ahead puro).
3. **Resultado** — ao sobrar uma peça, o jogo mostra tempo de inspeção, tempo de execução, número de lances e o seu *streak* de puzzles resolvidos de primeira.

Se a posição travar antes de você resolver (nenhuma captura possível com mais de uma peça no tabuleiro), é beco sem saída: `r` reinicia o puzzle. Reiniciar zera o streak — o jogo é one-shot de propósito.

## Controles

| Tecla         | Ação                                                        |
|---------------|-------------------------------------------------------------|
| Setas         | Pulam direto pra peça mais próxima naquela direção          |
| `Enter`       | Começa a resolver (na inspeção) / seleciona peça e captura  |
| `Esc`         | Cancela a seleção                                           |
| `h`           | Liga/desliga as dicas de captura                            |
| `n` / `p`     | Próximo / anterior puzzle                                   |
| `r`           | Reinicia o puzzle atual                                     |
| `1`–`0`       | Pula pro nível (1 a 10)                                     |
| `q`           | Sai                                                         |

A navegação não anda casa a casa: como você só quer pousar em peças (pra selecionar ou pra capturar), a seta leva o cursor direto pra peça ocupada mais próxima naquela direção.

## Níveis e puzzles

Os puzzles são gerados na hora, de trás pra frente: o gerador parte de uma solução válida e desfaz capturas, então toda posição que aparece tem solução garantida. São 10 níveis com 3 puzzles cada; o número de peças cresce com o nível (de 3 até 13), aumentando o tamanho da cadeia que você precisa enxergar.

## Requisitos de terminal

O jogo desenha as peças de dois jeitos:

- **Imagens (Kitty graphics protocol)** — peças renderizadas como imagens de verdade. Funciona em **kitty**, **Ghostty** e **WezTerm**, que são detectados automaticamente.
- **Glifos Unicode (♚♛♜♝♞♟)** — fallback pra qualquer outro terminal (gnome-terminal, iTerm2, Alacritty, Windows Terminal etc). Visual mais simples, mas totalmente jogável.

A detecção é automática, mas dá pra forçar:

```bash
cargo run -- --ascii    # força os glifos Unicode
cargo run -- --kitty    # força as imagens
```

Outros requisitos:

- Terminal de pelo menos **80 colunas × 46 linhas** (o jogo espera você redimensionar se estiver menor).
- Som de captura é opcional: se você tiver `ffplay`, `mpv`, `mpg123` ou `play` no PATH, toca um efeito a cada captura. Sem nenhum deles, o jogo funciona igual, só sem som.

## Build

```bash
git clone git@github.com:chagas42/solo_chess.git
cd solo_chess
cargo run
```

Precisa de Rust 1.85+ (edition 2024). A única dependência é o `crossterm` — base64, geração de puzzle, RNG e o protocolo do Kitty são todos implementados aqui no projeto.

## Estrutura

| Arquivo       | Responsabilidade                                                     |
|---------------|----------------------------------------------------------------------|
| `main.rs`     | Loop do jogo, regras do Solo Chess, render do tabuleiro, modo treino |
| `puzzle.rs`   | Geração reversa de puzzles e RNG                                     |
| `kitty.rs`    | Base64 e Kitty graphics protocol pra desenhar as peças               |

## Roadmap

- Modo escuro de verdade (esconder o tabuleiro depois da inspeção e jogar de memória)
- Janela de inspeção com limite de tempo, no estilo dos 15s do speedcubing
- Histórico de tempos e estatísticas por nível
- Distribuição: crates.io e binários prontos

## Contribuindo

Projeto em desenvolvimento — abra uma issue antes de mandar PR, a arquitetura ainda pode mudar.

## Licença

MIT
