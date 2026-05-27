<script lang="ts">
  import type { GameAction, GameState } from '$lib/api.svelte'
  import { resultTitle } from '$lib/hive-ui'

  let {
    currentGame,
    actionHistory,
    onClose,
    openHistory,
  }: {
    currentGame: GameState
    actionHistory: GameAction[]
    onClose: () => void
    openHistory: () => void | Promise<void>
  } = $props()

  const detail = $derived(resultDetail(currentGame, actionHistory))

  function resultDetail(game: GameState, history: GameAction[]) {
    const moves = history.filter((action) => action.type !== 'cannot_move').length
    const moveLabel = moves === 1 ? 'move' : 'moves'
    if (game.current_status === 'draw') return `Draw after ${moves} ${moveLabel}.`
    if (game.current_status === 'cancelled') return `Cancelled after ${moves} ${moveLabel}.`
    return `${resultTitle(game)} after ${moves} ${moveLabel}.`
  }
</script>

<div class="modal-backdrop" role="presentation" onclick={onClose}>
  <div
    class="modal-panel result-modal"
    role="dialog"
    aria-modal="true"
    aria-label="Game result"
    tabindex="-1"
    onclick={(event) => event.stopPropagation()}
    onkeydown={(event) => event.stopPropagation()}
  >
    <p class="eyebrow">Game over</p>
    <h2>{resultTitle(currentGame)}</h2>
    <p>{detail}</p>
    <div class="button-row">
      <button
        class="secondary"
        type="button"
        onclick={() => {
          onClose()
          void openHistory()
        }}
      >
        Show history
      </button>
      <button class="primary" type="button" onclick={onClose}>Close</button>
    </div>
  </div>
</div>
