# Plano de implementação — OrganizadorApp

> Este documento é a especificação completa do projeto, escrito para ser entregue a
> outra IA (ou desenvolvedor) executar a implementação. O esqueleto de código que
> acompanha este plano (pasta `src-tauri/` e `src/`) já define a estrutura de módulos,
> assinaturas de função e comentários `TODO` — a IA que implementar deve preencher
> esses `todo!()` seguindo exatamente as regras descritas aqui, sem mudar a arquitetura
> geral sem justificativa forte.

## 1. Visão geral do produto

Um aplicativo Windows portátil (`.exe` único, sem instalação) que:

1. O usuário abre o app e seleciona (ou arrasta) uma pasta.
2. O app varre recursivamente todos os arquivos e subpastas.
3. Um motor de classificação identifica o conteúdo real de cada arquivo (não confia
   em nome/extensão isoladamente) e agrupa tudo em categorias/tags **criadas em tempo
   real** — o app não vem com nenhuma taxonomia pré-definida.
4. O usuário vê uma **pré-visualização lado a lado** (estrutura atual vs. proposta)
   antes de qualquer mudança ser aplicada.
5. O usuário pode corrigir qualquer classificação com o botão direito, e essas
   correções alimentam um perfil de aprendizado local, permanente e portátil.
6. Só ao clicar em "Aplicar" os arquivos são de fato movidos — nunca deletados,
   sempre com log reversível (undo).

## 2. Requisitos funcionais (RF)

| ID | Requisito |
|----|-----------|
| RF01 | Selecionar pasta via diálogo nativo ou drag-and-drop |
| RF02 | Varrer recursivamente arquivos e subpastas, com barra de progresso em tempo real |
| RF03 | Detectar o tipo real de cada arquivo pelos bytes (magic number), não pela extensão declarada |
| RF04 | Extrair conteúdo textual representativo de PDF, DOCX, XLSX, TXT/MD/CSV |
| RF05 | Agrupar arquivos semanticamente parecidos mesmo com nomes/extensões diferentes |
| RF06 | Gerar nomes de categoria legíveis automaticamente (ex.: "Boletos de luz"), sem taxonomia fixa |
| RF07 | Exibir pré-visualização lado a lado (atual vs. proposta) antes de aplicar |
| RF08 | Permitir correção manual via clique direito: mudar categoria, criar tag, ignorar arquivo, criar regra permanente |
| RF09 | Aplicar mudanças movendo arquivos (nunca deletando), com log reversível |
| RF10 | Desfazer (undo) a última aplicação |
| RF11 | Gerenciar tags/categorias: renomear, mesclar, excluir |
| RF12 | Fazer backup e restaurar o perfil do usuário (tags, regras, correções) |
| RF13 | Alternar idioma: português brasileiro (padrão) e inglês americano |
| RF14 | Alternar tema: claro, escuro, ou seguir o sistema |
| RF15 | Aprendizado incremental: cada correção do usuário melhora classificações futuras, sem enviar nada para fora da máquina |

## 3. Requisitos não-funcionais (RNF)

| ID | Requisito |
|----|-----------|
| RNF01 | 100% offline — nenhum dado do usuário sai da máquina |
| RNF02 | Deve rodar fluido em máquinas com specs modestas (ex.: 4GB RAM, sem GPU dedicada) |
| RNF03 | `.exe` deve ser executável diretamente, sem instalador obrigatório, sem depender do navegador do usuário |
| RNF04 | Nunca deletar arquivos — apenas mover, sempre com trilha de auditoria |
| RNF05 | Nunca tocar em pastas de sistema por padrão (ver seção 10) |
| RNF06 | Perfil do usuário deve ser portátil (copiar a pasta do app = copiar o perfil) |
| RNF07 | UI deve permanecer responsiva durante varredura/classificação (processamento em paralelo/assíncrono, resultados incrementais) |

## 4. Arquitetura e stack

- **Backend/core**: Rust — performance, baixo consumo de memória, sem GC, paralelismo real via `rayon`.
- **Shell/empacotamento**: Tauri 2 — usa o WebView2 do Windows (não embute um Chromium inteiro como Electron), resultando em binário final pequeno (dezenas de MB, não centenas).
- **Frontend**: Svelte + TypeScript — reativo, leve, ótimo para a árvore de preview com muitos itens.
- **Banco local**: SQLite via `rusqlite` — um único arquivo `profile.db`, zero servidor.
- **IA local**:
  - Camada 2 (embeddings): modelo pequeno de sentence-embedding (ex.: variante quantizada de MiniLM) via `candle` ou ONNX Runtime.
  - Camada 3 (nomeação semântica): modelo de linguagem pequeno quantizado (ex.: Qwen2.5-1.5B-Instruct ou Phi-3-mini, formato GGUF, ~700MB–1GB), rodando via bindings de `llama.cpp`, 100% local, CPU-only.

Ver `src-tauri/Cargo.toml` para a lista completa de dependências sugeridas.

## 5. Estrutura do projeto (referência do esqueleto entregue)

```
src-tauri/src/
  main.rs                 → registra comandos Tauri, abre o banco ao lado do .exe
  commands/                → funções expostas ao frontend (scan, classify, apply, profile)
  engine/                  → motor de classificação em 3 camadas (o núcleo do produto)
  db/                      → schema.sql + modelos + abertura do banco
  fs_ops/                  → mover arquivos com segurança + log de undo
src/
  routes/                  → telas: FolderSelect, Preview, Settings, TagManager
  lib/api.ts               → wrappers de `invoke()` para os comandos Rust
  lib/i18n/                → pt-BR.json, en-US.json
```

## 6. Esquema de banco de dados

Ver `src-tauri/src/db/schema.sql` para o DDL completo. Resumo das tabelas:

- **files** — metadata de cada arquivo escaneado (path, extensão declarada vs. detectada, tamanho, datas, hash de conteúdo).
- **categories** — tags/categorias, com `created_by` (`auto` | `user`) para diferenciar o que a IA criou do que o usuário criou manualmente.
- **file_categories** — associação arquivo↔categoria, com `confidence` e `assigned_by` (`heuristic` | `embedding` | `llm` | `user`).
- **classification_rules** — regras aprendidas (padrão de nome, palavra-chave de conteúdo, extensão) → categoria, com `hit_count` e `confidence_weight` crescentes a cada acerto.
- **user_corrections** — toda correção manual, histórico completo (alimenta o aprendizado).
- **scan_sessions** — cada varredura, com status e contadores.
- **move_log** — toda operação de mover arquivo, com `undone` boolean (é a base do undo).
- **embeddings_cache** — vetor de embedding por `content_hash`, para nunca recalcular o mesmo conteúdo duas vezes.
- **settings** — chave-valor simples (idioma, tema, caminho do modelo local, etc.).

**Ponto crítico de precisão**: a tabela `classification_rules` é o que faz o app ficar
mais preciso com o tempo. Toda correção do usuário deve virar (ou reforçar) uma regra,
e a camada 1 (heurísticas) deve **sempre consultar essa tabela primeiro**, antes de
qualquer heurística genérica.

## 7. Motor de classificação — as 3 camadas (seção mais importante)

O objetivo é ser rápido E preciso ao mesmo tempo, então cada camada só processa o
subconjunto de arquivos que a camada anterior não resolveu com confiança.

### Camada 1 — Heurísticas instantâneas (100% dos arquivos)

Roda em memória, sem I/O pesado. Ordem de prioridade dos sinais:

1. **Regras aprendidas** (`classification_rules`) — se o arquivo bate com um padrão
   já conhecido (extensão + palavra no nome, ou regra criada manualmente pelo
   usuário), classifica direto com confiança alta.
2. **Nome legível** — se o nome do arquivo contém palavras reais (não é só número,
   hash ou timestamp de câmera), usar NLP leve (tokenização + stopwords pt/en) para
   already produzir um palpite.
3. **Extensão detectada vs. declarada** — usar a crate `infer` para ler os primeiros
   bytes do arquivo. Se a extensão declarada não bate com o tipo real, **isso já é
   sinal de baixa confiança** e força passagem para a camada 2.
4. **Contexto de pasta-pai** — se o arquivo já está dentro de uma pasta com nome
   significativo (ex.: "Notas Fiscais 2024"), usar isso como sinal adicional.

Se a confiança combinada for **≥ 0.75**, o arquivo já está classificado. Caso
contrário, marca `needs_deeper_analysis = true` e segue para a camada 2. Na prática,
arquivos com nomes tipo `8291.pdf`, `IMG_20240512_scan.jpg` ou `documento(3).pdf`
quase sempre caem aqui.

### Camada 2 — Conteúdo + embeddings (só arquivos ambíguos)

1. Extrair um **trecho representativo** do conteúdo — não o arquivo inteiro. Para
   PDF/DOCX: só as primeiras ~1–2 páginas ou ~2000 caracteres. Para XLSX: nomes das
   planilhas + primeiras linhas. Isso mantém a operação rápida mesmo em arquivos
   grandes.
2. Gerar um embedding local desse trecho.
3. Agrupar (clustering) arquivos com embeddings próximos — mesmo que os nomes de
   arquivo não tenham nada em comum entre si, arquivos com o **mesmo tipo de
   conteúdo** (ex.: vários boletos de luz com nomes aleatórios) caem no mesmo grupo.
4. Cachear o embedding por hash de conteúdo (`embeddings_cache`) — se o usuário rodar
   o app de novo sobre a mesma pasta, arquivos inalterados não são reprocessados.

### Camada 3 — Nomeação semântica via IA local (1x por cluster, não por arquivo)

Depois que a camada 2 formou os grupos, um modelo de linguagem local pequeno recebe
uma amostra de 3–5 trechos representativos do cluster e devolve um nome de categoria
curto e legível (ex.: "Boletos de luz", "Contratos de aluguel", "Fotos de viagem —
praia"). Esse é o passo mais caro computacionalmente, mas roda **uma vez por grupo**,
não por arquivo — é isso que mantém o app viável em máquinas leves mesmo processando
milhares de arquivos.

Prompt sugerido (few-shot, adaptado ao idioma ativo do app):
> "Aqui estão trechos de N arquivos parecidos: [amostras]. Em até 3 palavras, qual
> categoria/nome de pasta descreve melhor esse grupo? Responda só com o nome, sem
> explicação."

### Retroalimentação (fecha o ciclo)

Toda vez que o usuário corrige uma classificação na tela de preview (RF08), a
correção é gravada em `user_corrections` **e imediatamente** vira/reforça uma entrada
em `classification_rules`. Da próxima vez que um arquivo parecido aparecer, a camada 1
já resolve sozinha, sem precisar da camada 2/3 de novo. É assim que o app fica mais
rápido e mais preciso especificamente para o vocabulário e hábitos daquele usuário.

## 8. Fluxo de telas

1. **Seleção de pasta** — botão + área de drag-and-drop.
2. **Varredura** — barra de progresso (contagem de arquivos, tamanho total).
3. **Classificação** — progresso incremental; a tela de preview já pode começar a
   popular resultados conforme eles chegam, sem esperar o processamento inteiro
   terminar.
4. **Pré-visualização** — duas colunas (estrutura atual / proposta), tags como
   chips coloridos. Clique direito em qualquer item abre: mudar categoria, criar
   nova tag, ignorar este arquivo, "sempre classificar assim" (vira regra
   permanente).
5. **Aplicar** — só move ao clicar no botão; confirmação explícita.
6. **Gerenciador de tags** — acessível a qualquer momento: renomear, mesclar,
   excluir tags, ver quantos arquivos cada uma tem.
7. **Configurações** — idioma, tema, caminho do modelo de IA local, backup/restauração
   de perfil.

## 9. Segurança e não-destrutividade

- **Nunca deletar** — só mover. Se dois arquivos colidirem no destino, renomear com
  sufixo, nunca sobrescrever.
- **Excluir por padrão**: `Windows/`, `Program Files/`, `Program Files (x86)/`,
  `AppData/`, `.git/`, `node_modules/`, arquivos de sistema ocultos. O usuário pode
  desmarcar essa proteção nas configurações avançadas, mas o padrão é conservador.
- **Log de tudo antes de mover** — `move_log` é gravado antes da operação de fato, de
  forma que mesmo um crash no meio do processo deixa rastro suficiente para
  reconciliar/desfazer.
- **Undo real** — reverte usando o `move_log` da última sessão de aplicação.

## 10. Empacotamento portátil (ponto que já discutimos na conversa)

- Configurar `webviewInstallMode` como `fixedRuntime` no `tauri.conf.json`, embutindo
  o runtime do WebView2 — o app funciona sozinho mesmo em Windows sem o WebView2
  pré-instalado.
- Para a experiência de "baixei, arrastei, abri" (sem instalador), usar diretamente o
  binário compilado (`target/release/organizador-app.exe`), não o instalador NSIS —
  o NSIS é opcional, só necessário se algum dia quiser atalho no menu iniciar.
- O banco (`data/profile.db`) e configurações ficam sempre ao lado do `.exe`
  (`Database::open_beside_executable`, já no esqueleto) — mover a pasta inteira leva
  o perfil junto, viabilizando o backup/restauração pedido (RF12).
- O modelo de IA local (camada 3, ~700MB–1GB) **não deve ser embutido no binário
  inicial** — baixar sob demanda no primeiro uso, para manter o download inicial
  leve. Se preferir zero downloads pós-instalação, empacotar junto e aceitar um
  instalador maior — decisão de produto, não técnica.

## 11. Internacionalização e tema

- `svelte-i18n` com `src/lib/i18n/pt-BR.json` (padrão) e `en-US.json`.
- Detectar idioma do Windows no primeiro uso; salvar escolha em `settings`.
- Tema via variáveis CSS, com opção "seguir o sistema" usando
  `prefers-color-scheme` + fallback manual.

## 12. Plano de fases

**Fase 0 — MVP funcional (foco: provar o pipeline fim a fim)**
- Varredura + camada 1 (heurísticas) apenas, sem embeddings/LLM ainda.
- Preview simples (sem drag visual sofisticado) + aplicar + undo básico.
- Sem tags manuais ainda, sem backup de perfil.

**Fase 1 — Precisão**
- Implementar camada 2 (extração de conteúdo + embeddings + clustering).
- Implementar camada 3 (nomeação via IA local), com download do modelo sob demanda.
- Cache de embeddings por hash de conteúdo.

**Fase 2 — Aprendizado e gestão**
- Correções via botão direito alimentando `classification_rules`.
- Gerenciador de tags completo (renomear, mesclar, excluir).
- Backup/restauração de perfil (export/import do zip da pasta `data/`).

**Fase 3 — Polimento**
- i18n completo, tema claro/escuro, empacotamento final com WebView2 fixed,
  testes de performance em máquina de specs baixas.

## 13. Critérios de aceite / testes de precisão sugeridos

Montar pastas de teste sintéticas e validar taxa de acerto:

1. Pasta com 50 PDFs de boletos de luz/água/internet, todos com nomes numéricos
   aleatórios → esperado: agrupados corretamente em poucas categorias coerentes
   (ex.: "Boletos", ou separados por tipo de conta).
2. Pasta de "Downloads" real e bagunçada (instaladores, imagens, PDFs, planilhas,
   zips misturados) → esperado: nenhuma pasta de sistema tocada, categorias fazem
   sentido, arquivos ambíguos (tipo `setup(1).exe`) tratados com cautela.
3. Pasta de fotos com nomes de câmera (`IMG_XXXX.jpg`) misturadas com nomes
   descritivos → esperado: agrupamento por conteúdo visual/data quando não há texto,
   sem inventar categorias contraditórias entre execuções.
4. Repetir a varredura da mesma pasta sem alterações → esperado: resultado
   idêntico e rápido (cache de embeddings evitando reprocessamento).
5. Corrigir manualmente 5 classificações erradas e rodar de novo em uma pasta
   parecida → esperado: as correções influenciam os novos resultados.

## 14. Riscos conhecidos e mitigação

| Risco | Mitigação |
|---|---|
| Modelo local de IA pesado demais para máquina fraca | Camada 3 roda por cluster, não por arquivo; permitir desativá-la nas configurações, caindo só nas camadas 1 e 2 |
| Falso positivo movendo arquivo importante para lugar errado | Preview obrigatório antes de aplicar + undo sempre disponível + nunca deletar |
| WebView2 ausente em Windows corporativo/LTSC | `fixedRuntime` embutido no bundle, elimina a dependência externa |
| Extração de PDF/DOCX corrompido travando a varredura | Toda extração de conteúdo deve rodar com timeout e `catch_unwind`/`Result`, nunca derrubar o processo principal |
| Clustering instável (mesma pasta gera categorias diferentes em execuções distintas) | Usar seed fixa no algoritmo de clustering e priorizar sempre `classification_rules` existentes antes de reclusterizar do zero |
