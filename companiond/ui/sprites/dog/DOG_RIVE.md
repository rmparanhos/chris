# Rig do cachorro no Rive (respiração de verdade)

O CSS não consegue deformar **só o peito** de um PNG achatado — por isso a
respiração ficava ruim. A solução é um arquivo **Rive** (`.riv`) com *mesh*
(deforma regiões da imagem) e uma *state machine* ligada aos estados do CHRIS.

O código já está pronto: assim que existir o arquivo
`companiond/ui/sprites/dog/dog.riv`, o app passa a renderizar o cachorro pelo
Rive automaticamente (e esconde os PNGs). Se o arquivo não existir, ele continua
usando os PNGs — então nada quebra enquanto você monta o rig.

## Contrato que o `.riv` precisa seguir

O `dog-rive.js` espera:

1. **Uma State Machine** — a **primeira** do arquivo é a usada (pode deixar o
   nome padrão `State Machine 1`).
2. **Um input do tipo Number chamado exatamente `state`**, com a convenção:

   | valor | estado     |
   |------:|------------|
   |   0   | idle       |
   |   1   | alert      |
   |   2   | approved   |
   |   3   | denied     |
   |   4   | pr         |

Só isso. O resto (qual animação toca em cada valor) é com você no editor.

## Passo a passo no editor (rive.app — grátis)

1. Crie uma conta em https://rive.app e abra o **editor** (web).
2. **New File** → crie um **Artboard** com proporção quadrada (combina com os
   sprites 256×256).
3. **Importe** `idle.png` (arraste pro artboard). Centralize e ajuste o tamanho.
4. Selecione a imagem → **Create Mesh**: adicione vértices contornando o corpo,
   com vértices extras na **região do peito/tórax**.
5. Adicione **bones** (ou use os vértices direto): um osso/controle na altura do
   peito é o que vai "inflar".
6. **Animações** (aba Animate):
   - `idle`: animação em **loop** — anime o peito expandindo/contraindo de leve
     (mexa os vértices/bone do tórax ~3–5%, ~3–4s, ease-in-out), mantendo as
     **patas paradas**. Esse é o efeito que faltava.
   - `alert` / `approved` / `denied` / `pr`: animações curtas (pode ser um leve
     "wobble", "hop", "shake" etc.) — ou só troque a cor/expressão se preferir.
     Não precisa ser elaborado.
7. **State Machine** (aba Animate → State Machine):
   - Crie a state machine.
   - Adicione o input **Number** chamado **`state`**.
   - Crie um estado para cada animação e **transições** baseadas em `state`:
     `state == 0` → idle (loop), `== 1` → alert, `== 2` → approved,
     `== 3` → denied, `== 4` → pr. Deixe idle como estado inicial.
8. **Export** → **Download** → salve como **`dog.riv`** dentro de
   `companiond/ui/sprites/dog/dog.riv`.
9. Gere o instalador de novo (Actions → Release) e pronto: o cachorro respira
   pelo peito e reage a cada estado.

## Dicas

- Quer reusar as expressões já desenhadas? Importe também `alert.png`,
  `approved.png` etc. e troque a visibilidade da imagem dentro de cada estado da
  state machine (assim mantém a arte por estado + a respiração por mesh).
- Mantenha a deformação **sutil** — respiração é discreta.
- O runtime é offline: o `.wasm` já está vendorado em `ui/vendor/rive.wasm`,
  não precisa de internet em runtime.
</content>
