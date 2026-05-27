<script lang="ts">
  import type { GameAction } from '$lib/api.svelte'
  import { actionSummary } from '$lib/hive-ui'

  let {
    gameId,
    actionHistory,
    onClose,
  }: {
    gameId: number
    actionHistory: GameAction[]
    onClose: () => void
  } = $props()
</script>

<div class="modal-backdrop" role="presentation" onclick={onClose}>
  <div
    class="modal-panel history-modal"
    role="dialog"
    aria-modal="true"
    aria-label="Game history"
    tabindex="-1"
    onclick={(event) => event.stopPropagation()}
    onkeydown={(event) => event.stopPropagation()}
  >
    <div class="modal-heading">
      <div>
        <p class="eyebrow">Game #{gameId}</p>
        <h2>History</h2>
      </div>
      <button class="ghost" type="button" onclick={onClose}>Close</button>
    </div>

    <div class="history-list full">
      {#each actionHistory.toReversed() as action}
        <div>
          <strong>#{action.move_number}</strong>
          <span>{actionSummary(action)}</span>
        </div>
      {:else}
        <div>
          <strong>-</strong>
          <span>No actions yet</span>
        </div>
      {/each}
    </div>
  </div>
</div>
