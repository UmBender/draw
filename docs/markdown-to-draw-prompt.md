# Prompt: Converter Markdown com grafos (Mermaid) para `.draw`

Copie e cole o prompt abaixo em uma LLM, substituindo `<COLE SEU MARKDOWN AQUI>` pelo seu conteúdo:

```text
Converta o conteúdo Markdown abaixo (incluindo blocos de grafo como Mermaid) diretamente para o formato de arquivo `.draw` (`draw-file 1`). Responda apenas com o conteúdo bruto do arquivo `.draw`, sem explicações nem blocos de código ao redor.

Regras do formato `.draw`:
1. Cabeçalho e página única:
draw-file 1
[pages]
current 0
count 1
[page]
index 0
camera 0.0 0.0 1.0
[document]
next-id <TOTAL_DE_FORMAS>

2. Cada forma fica em uma linha `shape <id> ...`, com `<id>` sequencial começando em `0` (`0`, `1`, ..., `next-id - 1`).
3. `<color_id>` é um inteiro de `0` a `5` (use `0` como cor padrão) e `<width>` é a espessura em pixels (use `3.0`). Coordenadas são números decimais (ex.: `100.0`).
4. Tipos de forma permitidos:
- Texto (títulos, notas, fórmulas `$...$` e rótulos de arestas):
  `shape <id> text <color_id> <width> <x> <y> <size_px> "<texto>"`
  Use tamanhos maiores para títulos (`#` -> `36.0`, `##` -> `28.0`, `###` -> `22.0`) e `18.0` para texto comum.
- Retângulo (nó de grafo):
  `shape <id> rect <color_id> <width> <x1> <y1> <x2> <y2> <fill_opt> - <text_opt>`
- Elipse / círculo (nó redondo/estado):
  `shape <id> ellipse <color_id> <width> <x1> <y1> <x2> <y2> <fill_opt> - <text_opt>`
  Em `rect` e `ellipse`, `<fill_opt>` é `-` (sem preenchimento) ou `<color_id>`; `<text_opt>` é `-` ou `<size_px> "<texto>"` (ex.: `18.0 "Início"`).
- Seta (aresta direcionada `-->`):
  `shape <id> arrow <color_id> <width> <x1> <y1> <x2> <y2>`
- Linha (aresta sem seta `---`):
  `shape <id> line <color_id> <width> <x1> <y1> <x2> <y2>`

5. Regras de conversão e layout:
- Ignore formatação Markdown não suportada (negrito, itálico, links, imagens, listas etc.), mantendo apenas o texto limpo, títulos e expressões TeX `$...$`.
- Escape aspas duplas como `\"` e quebras de linha como `\n` dentro de `"..."`.
- Posicione os nós dos grafos em uma grade legível (sem sobreposição, espaçamento mínimo de ~80 px entre caixas) e conecte os centros das bordas dos nós com `arrow` ou `line`. Se a aresta tiver rótulo, coloque um `text` próximo ao ponto médio da seta.

Conteúdo Markdown:
<COLE SEU MARKDOWN AQUI>
```
