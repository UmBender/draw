# Prompt: Converter Markdown (grafos, pontos no plano, tabelas/DP, segtrees) para a sintaxe suportada pelo `draw`

Copie e cole o prompt abaixo em uma LLM, substituindo `<COLE SEU MARKDOWN AQUI>` pelo seu conteúdo:

```text
Converta o conteúdo Markdown abaixo para um formato compatível com a ferramenta `draw`, transformando diagramas (como Mermaid), pontos no plano, tabelas/matrizes de DP e árvores de segmentos em blocos de texto na sintaxe nativa suportada pelo `draw`. Responda apenas com o conteúdo convertido.

Regras das sintaxes suportadas pelo `draw`:

1. Grafos e Árvores (remova cercas ```mermaid e estilos visuais como `((A))`, `[texto]`, `subgraph`, `style`):
   Comece o bloco com uma linha de palavra-chave:
   - `graph:` (grafo não direcionado)
   - `dgraph:` (grafo direcionado)
   - `wgraph:` / `wdgraph:` (grafos com pesos/rótulos nas arestas)
   - `tree:` / `wtree:` (árvores)
   - `parents:` / `parents0:` (lista de pais, indexada em 1 ou 0; `-1` para raiz)
   Nas linhas seguintes, escreva uma aresta por linha:
   - `u v` (aresta simples)
   - `u v tag` (aresta com peso/rótulo `tag`)
   - Também são aceitos `u-v` (não direcionada) e `u>v` (direcionada).
   Use para os vértices **apenas números inteiros não negativos** (`1`, `2`, `3`) ou **fórmulas curtas entre cifrões** (`$a$`, `$b$`, `$s$`, `$u$`, `$t$`). Não use letras soltas sem `$...$` (o parser rejeita `a b`, mas aceita `$a$ $b$` ou `1 2`). Pesos/rótulos de aresta também devem ser números ou `$...$`.

   Exemplo (Mermaid -> `draw`):
   dgraph:
   $a$ $b$ 5
   $b$ $c$

2. Árvore de Segmentos (`segtree:`):
   Para uma árvore de segmentos de tamanho `N` (`1` a `100`), escreva em uma única linha:
   segtree: N

3. Pontos em um Plano Cartesiano:
   Escreva um ponto por linha no formato:
   - `x y`
   - `x y rotulo` (tudo após os dois números vira o rótulo do ponto)
   Exemplo:
   0 0 O
   2 3 A
   -1 4.5 B

4. Tabelas, Matrizes e Tabelas de DP:
   Converta tabelas Markdown (`| a | b |`) removendo as barras `|` e a linha separadora (`|---|---|`).
   Escreva cada linha da tabela em uma linha, com as células separadas por espaço em branco (use `-` ou `0` para células vazias, sem espaços internos em cada célula).
   Exemplo:
   dp 0 1 2
   0 0 1 2
   1 1 2 3

5. Texto geral, Títulos e Fórmulas:
   - Mantenha títulos (`#`, `##`, `###`...), parágrafos simples e fórmulas TeX (`$...$`).
   - Remova formatações não suportadas (negrito, itálico, links, imagens, etc.), preservando apenas o texto limpo.

Conteúdo Markdown:
<COLE SEU MARKDOWN AQUI>
```
