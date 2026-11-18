# Prompt: Converter Markdown com grafos (Mermaid) para a sintaxe suportada pelo `draw`

Copie e cole o prompt abaixo em uma LLM, substituindo `<COLE SEU MARKDOWN AQUI>` pelo seu conteúdo:

```text
Converta o conteúdo Markdown abaixo para um Markdown compatível com a ferramenta `draw`, convertendo todos os blocos de grafo (como Mermaid) para blocos de texto na sintaxe nativa de grafos do `draw`. Responda apenas com o Markdown convertido.

Regras da sintaxe de grafos suportada pelo `draw`:
1. Cada grafo deve ser um bloco de texto simples (sem cercas ```mermaid), começando com uma linha de palavra-chave:
   - `graph:` (grafo não direcionado)
   - `dgraph:` (grafo direcionado)
   - `wgraph:` / `wdgraph:` (grafos com pesos/rótulos nas arestas)
   - `tree:` / `wtree:` (árvores)
   - `parents:` / `parents0:` (lista de pais, indexada em 1 ou 0)

2. Nas linhas seguintes, escreva uma aresta por linha no formato:
   - `u v` (aresta simples entre os nós `u` e `v`)
   - `u v tag` (aresta entre `u` e `v` com peso ou rótulo `tag`)
   - Também são aceitos `u-v` (não direcionada) e `u>v` (direcionada).

3. Use nomes simples para os vértices (números como `1`, `2`, `3` ou identificadores curtos como `a`, `b`, `s`, `u`, `t`). Remova estilos/formas visuais específicos do Mermaid (como `((A))`, `[texto]`, `subgraph`, `style`, etc.).

4. Exemplo de conversão:
   De Mermaid:
   ```mermaid
   graph LR
       A((a)) -->|5| B((b))
       B --> C((c))
   ```
   Para `draw`:
   dgraph:
   a b 5
   b c

5. Para o restante do Markdown fora dos grafos:
   - Mantenha títulos (`#`, `##`, `###`...), texto simples e fórmulas TeX (`$...$`).
   - Remova formatações não suportadas (negrito, itálico, links, imagens, etc.), preservando apenas o texto limpo.

Conteúdo Markdown:
<COLE SEU MARKDOWN AQUI>
```
