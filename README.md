# solo_chess

Um port do [Solo Chess do chess.com](https://www.chess.com/solo-chess) pra rodar no terminal, escrito em Rust.

No Solo Chess você joga sozinho. Todo lance tem que ser uma captura, cada peça captura no máximo duas vezes, e se tiver um rei ele precisa sobrar por último. O puzzle acaba quando sobra uma peça só. É um treino enxuto de look-ahead: você monta a sequência inteira de capturas de cabeça antes de tocar na primeira peça.

Fiz isso porque jogava bastante no chess.com e queria uma versão sem distração, só teclado, no terminal. A ideia também é servir como treino pra quem pratica visualização. No xadrez ele exercita o cálculo direto, que é enxergar uma sequência forçada antes de executar. Pra quem faz speedcubing, o que transfere é o hábito de planejar a sequência toda e mantê-la na memória antes de executar.

## Modo treino

Cada puzzle passa por três fases, pensadas pra você planejar em vez de resolver por tentativa e erro.

1. Inspeção. O tabuleiro aparece e o cronômetro de inspeção começa a rodar. Você estuda a posição e planeja a cadeia. Quando estiver pronto, `Enter` começa a resolver.
2. Resolução. O cronômetro de execução roda enquanto você joga. Por padrão as casas de captura aparecem como pontinhos. Aperte `h` pra desligar as dicas e resolver sem nenhuma ajuda.
3. Resultado. Quando sobra uma peça, o jogo mostra o tempo de inspeção, o tempo de execução, quantos lances você fez e seu streak de puzzles resolvidos de primeira.

Se a posição travar antes de você resolver, quando não há mais captura possível e ainda sobra mais de uma peça no tabuleiro, é um beco sem saída e o `r` reinicia. Reiniciar zera o streak, porque o jogo é one-shot de propósito.

## Controles

| Tecla         | Ação                                                        |
|---------------|-------------------------------------------------------------|
| Setas         | Pulam direto pra peça mais próxima naquela direção          |
| `Enter`       | Começa a resolver (na inspeção) ou seleciona peça e captura |
| `Esc`         | Cancela a seleção                                           |
| `h`           | Liga e desliga as dicas de captura                          |
| `n` / `p`     | Próximo e anterior puzzle                                   |
| `r`           | Reinicia o puzzle atual                                     |
| `1`-`0`       | Pula pro nível (1 a 10)                                     |
| `q`           | Sai                                                         |

A navegação não anda casa a casa. Como você só quer parar em cima de peças, seja pra selecionar ou pra capturar, a seta já leva o cursor direto pra peça mais próxima naquela direção.

## Níveis e puzzles

Os puzzles são gerados na hora, de trás pra frente. O gerador parte de uma solução válida e vai desfazendo capturas, então toda posição que aparece tem solução garantida. São 10 níveis com 3 puzzles cada. O número de peças cresce conforme o nível, de 3 até 13, e a cadeia que você precisa enxergar vai ficando maior.

## Requisitos de terminal

O jogo desenha as peças de dois jeitos.

- Imagens, via Kitty graphics protocol. As peças são renderizadas como imagens reais. Funciona no kitty, no Ghostty e no WezTerm, detectados automaticamente.
- Glifos Unicode (♚♛♜♝♞♟), o fallback pra qualquer outro terminal como gnome-terminal, iTerm2, Alacritty ou Windows Terminal. Visual mais simples, mas igualmente jogável.

A detecção é automática, mas dá pra forçar:

```bash
cargo run -- --ascii    # força os glifos Unicode
cargo run -- --kitty    # força as imagens
```

Fora isso:

- Terminal de pelo menos 80x46. Se estiver menor, o jogo pede pra aumentar a janela.
- O som de captura é opcional. Se você tiver `ffplay`, `mpv`, `mpg123` ou `play` no PATH, toca um efeito a cada captura. Sem nenhum deles o jogo funciona normalmente, só sem som.

## Build

```bash
git clone git@github.com:chagas42/solo_chess.git
cd solo_chess
cargo run
```

Precisa de Rust 1.85+ (edition 2024). A única dependência é o `crossterm`. Base64, geração de puzzle, RNG e o protocolo do Kitty estão todos implementados no próprio projeto.

## Estrutura

| Arquivo       | Responsabilidade                                                     |
|---------------|----------------------------------------------------------------------|
| `main.rs`     | Loop do jogo, regras do Solo Chess, render do tabuleiro, modo treino |
| `puzzle.rs`   | Geração reversa de puzzles e RNG                                     |
| `kitty.rs`    | Base64 e Kitty graphics protocol pra desenhar as peças               |

## Roadmap

- Modo cego, escondendo o tabuleiro depois da inspeção pra jogar de memória
- Janela de inspeção com tempo limite, no estilo dos 15 segundos do speedcubing
- Histórico de tempos e estatísticas por nível
- Distribuição em binários prontos e na crates.io

## Contribuindo

Projeto ainda em desenvolvimento. Abre uma issue antes de mandar PR, porque a arquitetura ainda pode mudar.

## Licença

MIT
