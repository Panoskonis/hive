<script lang="ts">
  import type { GameAction, GameState, PieceType, PlayerColor } from '$lib/api.svelte'
  import { pieceAssets, pieceTypes, playerColors, statusText } from '$lib/hive-ui'

  let {
    currentGame,
    actionHistory,
    isSelfPlay,
    isMyTurn,
    activePlayerColor,
    selectedHandPiece,
    showCoordinates,
    soundMuted,
    playError,
    canUseInventory,
    selectHandPiece,
    openHistory,
    toggleCoordinates,
    toggleSoundMuted,
  }: {
    currentGame: GameState
    actionHistory: GameAction[]
    isSelfPlay: boolean
    isMyTurn: boolean
    activePlayerColor: PlayerColor | null
    selectedHandPiece: PieceType | null
    showCoordinates: boolean
    soundMuted: boolean
    playError: string
    canUseInventory: (color: PlayerColor) => boolean
    selectHandPiece: (piece: PieceType, color: PlayerColor) => void
    openHistory: () => void | Promise<void>
    toggleCoordinates: () => void
    toggleSoundMuted: () => void
  } = $props()
</script>

<aside class="play-panel">
  <div>
    <p class="eyebrow">Game #{currentGame.id}</p>
    <h1>{statusText(currentGame.current_status)}</h1>
  </div>

  <div class="turn-grid">
    <div>
      <span>Turn</span>
      <strong>{currentGame.current_turn}</strong>
    </div>
    <div>
      <span>You</span>
      <strong>{isSelfPlay ? 'both' : (currentGame.viewer_color ?? 'observer')}</strong>
    </div>
    <div>
      <span>Move</span>
      <strong>{currentGame.move_number}</strong>
    </div>
    <div>
      <span>Actions</span>
      <strong>{actionHistory.length}</strong>
    </div>
  </div>

  {#if currentGame.current_status === 'waiting_for_opponent' && currentGame.invite_code}
    <section class="invite-panel" aria-label="Game invite code">
      <span>Invite code</span>
      <code>{currentGame.invite_code}</code>
    </section>
  {/if}

  <section class="hand-panel">
    <div class="panel-heading tight">
      <h2>Inventories</h2>
      <span>{isMyTurn ? `${currentGame.current_turn} to play` : 'Waiting'}</span>
    </div>

    <div class="hand-inventories">
      {#each playerColors as color}
        <div class={`inventory-column owner-${color}`} class:playable={canUseInventory(color)}>
          <div class="inventory-heading">
            <strong>{color}</strong>
          </div>

          <div class="hand-grid">
            {#each pieceTypes as piece}
              <button
                class={`hand-piece hand-${piece}`}
                class:active={selectedHandPiece === piece && activePlayerColor === color}
                class:empty={currentGame.inventories[color][piece] === 0}
                type="button"
                onclick={() => selectHandPiece(piece, color)}
                disabled={!canUseInventory(color) || currentGame.inventories[color][piece] === 0}
                title={`${color} ${piece}`}
              >
                <img src={pieceAssets[piece]} alt="" />
                <strong>{currentGame.inventories[color][piece]}</strong>
              </button>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </section>

  <button class="secondary history-button" type="button" onclick={openHistory}>
    Show history
    <span>{actionHistory.length}</span>
  </button>

  <button
    class="secondary coordinate-toggle"
    class:active={showCoordinates}
    type="button"
    aria-pressed={showCoordinates}
    onclick={toggleCoordinates}
  >
    Coordinates
    <span>{showCoordinates ? 'On' : 'Off'}</span>
  </button>

  <button
    class="secondary sound-toggle"
    class:active={!soundMuted}
    type="button"
    aria-pressed={!soundMuted}
    onclick={toggleSoundMuted}
  >
    Sound
    <span>{soundMuted ? 'Off' : 'On'}</span>
  </button>

  {#if playError}
    <p class="error">{playError}</p>
  {/if}
</aside>
