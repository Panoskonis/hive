<script lang="ts">
  import { onMount } from 'svelte'
  import {
    ApiClient,
    type BoardCell,
    type Game,
    type GameAction,
    type GameState,
    type PieceType,
    type Position,
  } from './lib/api.svelte'

  const api = new ApiClient()
  const boardRadius = 5
  const pieceTypes: PieceType[] = ['queen', 'ant', 'beetle', 'grasshopper', 'spider', 'mosquito', 'ladybug', 'pillbug']
  const pieceLabels: Record<PieceType, string> = {
    queen: 'Q',
    ant: 'A',
    beetle: 'B',
    grasshopper: 'G',
    spider: 'S',
    mosquito: 'M',
    ladybug: 'L',
    pillbug: 'P',
  }

  let view = $state<'landing' | 'login' | 'register' | 'dashboard' | 'play'>(
    api.isAuthenticated ? 'dashboard' : 'landing',
  )
  let authMode = $state<'login' | 'register'>('login')
  let username = $state('')
  let email = $state('')
  let password = $state('')
  let authError = $state('')
  let authBusy = $state(false)

  let creatorColor = $state<'white' | 'black'>('white')
  let mosquitoEnabled = $state(false)
  let ladybugEnabled = $state(false)
  let pillbugEnabled = $state(false)
  let inviteCode = $state('')
  let games = $state<Game[]>([])
  let selectedInvite = $state<Game | null>(null)
  let gameError = $state('')
  let gameBusy = $state(false)
  let gameLoadBusy = $state(false)
  let currentGame = $state<GameState | null>(null)
  let playError = $state('')
  let selectedHandPiece = $state<PieceType | null>(null)
  let selectedPosition = $state<string | null>(null)
  let showHistoryModal = $state(false)
  let showResultModal = $state(false)
  let pollTimer: number | null = null

  const waitingGames = $derived(games.filter((game) => game.current_status === 'waiting_for_opponent'))
  const activeGames = $derived(games.filter((game) => game.current_status === 'in_progress'))
  const isMyTurn = $derived(
    currentGame?.viewer_color !== null && currentGame?.viewer_color === currentGame?.current_turn,
  )
  const boardCells = $derived(currentGame ? playableCells(currentGame) : [])
  const viewBox = $derived(boardViewBox(boardCells))

  onMount(() => {
    if (api.isAuthenticated) {
      void refreshGames()
    }

    return () => stopPolling()
  })

  function showAuth(mode: 'login' | 'register') {
    authMode = mode
    view = mode
    authError = ''
  }

  async function submitAuth() {
    authBusy = true
    authError = ''

    try {
      if (authMode === 'login') {
        await api.login(email, password)
      } else {
        await api.register(username, email, password)
      }
      password = ''
      view = 'dashboard'
      await refreshGames()
    } catch (error) {
      authError = readableError(error)
    } finally {
      authBusy = false
    }
  }

  async function logout() {
    stopPolling()
    await api.logout()
    games = []
    selectedInvite = null
    currentGame = null
    showHistoryModal = false
    showResultModal = false
    view = 'landing'
  }

  async function refreshGames() {
    if (!api.isAuthenticated) return
    gameLoadBusy = true
    gameError = ''

    try {
      games = await api.listGames()
    } catch (error) {
      gameError = readableError(error)
    } finally {
      gameLoadBusy = false
    }
  }

  async function createGame() {
    gameBusy = true
    gameError = ''

    try {
      const game = await api.createGame({
        creator_color: creatorColor,
        mosquito_enabled: mosquitoEnabled,
        ladybug_enabled: ladybugEnabled,
        pillbug_enabled: pillbugEnabled,
      })
      await refreshGames()
      inviteCode = game.invite_code ?? ''
    } catch (error) {
      gameError = readableError(error)
    } finally {
      gameBusy = false
    }
  }

  async function previewInvite() {
    gameBusy = true
    gameError = ''
    selectedInvite = null

    try {
      selectedInvite = await api.previewInvite(inviteCode)
    } catch (error) {
      gameError = readableError(error)
    } finally {
      gameBusy = false
    }
  }

  async function joinInvite() {
    gameBusy = true
    gameError = ''

    try {
      const game = await api.joinGame(inviteCode)
      await refreshGames()
      selectedInvite = null
      inviteCode = ''
      await openGame(game.id)
    } catch (error) {
      gameError = readableError(error)
    } finally {
      gameBusy = false
    }
  }

  function readableError(error: unknown) {
    return error instanceof Error ? error.message : 'Something went wrong'
  }

  async function openGame(gameId: number) {
    stopPolling()
    playError = ''
    selectedHandPiece = null
    selectedPosition = null
    showHistoryModal = false
    showResultModal = false

    try {
      setCurrentGame(await api.getGameState(gameId))
      view = 'play'
      startPolling()
    } catch (error) {
      playError = readableError(error)
      gameError = playError
    }
  }

  function leaveGame() {
    stopPolling()
    currentGame = null
    selectedHandPiece = null
    selectedPosition = null
    showHistoryModal = false
    showResultModal = false
    view = 'dashboard'
    void refreshGames()
  }

  function startPolling() {
    stopPolling()
    if (!currentGame || currentGame.current_status !== 'in_progress') return
    pollTimer = window.setInterval(async () => {
      if (!currentGame || currentGame.current_status !== 'in_progress') {
        stopPolling()
        return
      }

      try {
        const nextGame = await api.getGameState(currentGame.id)
        setCurrentGame(nextGame)
        if (nextGame.current_status !== 'in_progress') {
          stopPolling()
          await refreshGames()
        }
      } catch (error) {
        playError = readableError(error)
      }
    }, 2000)
  }

  function stopPolling() {
    if (pollTimer !== null) {
      window.clearInterval(pollTimer)
      pollTimer = null
    }
  }

  function selectHandPiece(piece: PieceType) {
    if (!currentGame || !isMyTurn || currentGame.inventories[currentGame.viewer_color!][piece] <= 0) return
    selectedHandPiece = selectedHandPiece === piece ? null : piece
    selectedPosition = null
  }

  function selectBoardCell(cell: BoardCell) {
    if (!currentGame || !isMyTurn) return
    const key = coordKey(cell)
    const topPiece = cell.pieces.at(-1)
    if (legalFromActions(key).length > 0 || topPiece?.color === currentGame.current_turn) {
      selectedPosition = selectedPosition === key ? null : key
      selectedHandPiece = null
    }
  }

  async function clickBoardCell(cell: BoardCell) {
    if (!currentGame || !isMyTurn) return
    const placement = selectedHandPiece
      ? currentGame.legal_actions.find(
          (action) =>
            action.type === 'place' && action.piece_type === selectedHandPiece && coordKey(action.to) === coordKey(cell),
        )
      : null
    if (placement) {
      await submitGameAction(placement)
      return
    }

    if (selectedPosition) {
      const move = currentGame.legal_actions.find(
        (action) =>
          (action.type === 'move' || action.type === 'pillbug_special') &&
          coordKey(action.from) === selectedPosition &&
          coordKey(action.to) === coordKey(cell),
      )
      if (move) {
        await submitGameAction(move)
        return
      }
    }

    if (cell.pieces.length > 0) {
      selectBoardCell(cell)
    }
  }

  async function submitGameAction(action: GameAction) {
    if (!currentGame) return
    playError = ''

    try {
      setCurrentGame(await api.submitAction(currentGame.id, action))
      selectedHandPiece = null
      selectedPosition = null
      await refreshGames()
      startPolling()
    } catch (error) {
      playError = readableError(error)
    }
  }

  function legalFromActions(key: string) {
    return (
      currentGame?.legal_actions.filter(
        (action) =>
          (action.type === 'move' || action.type === 'pillbug_special') && coordKey(action.from) === key,
      ) ?? []
    )
  }

  function isHighlighted(cell: BoardCell) {
    if (!currentGame) return false
    const key = coordKey(cell)
    if (selectedHandPiece) {
      return currentGame.legal_actions.some(
        (action) => action.type === 'place' && action.piece_type === selectedHandPiece && coordKey(action.to) === key,
      )
    }
    if (selectedPosition) {
      return currentGame.legal_actions.some(
        (action) =>
          (action.type === 'move' || action.type === 'pillbug_special') &&
          coordKey(action.from) === selectedPosition &&
          coordKey(action.to) === key,
      )
    }
    return false
  }

  function setCurrentGame(nextGame: GameState) {
    const previousStatus = currentGame?.current_status
    currentGame = nextGame
    if (isFinishedStatus(nextGame.current_status) && previousStatus !== nextGame.current_status) {
      showResultModal = true
    }
  }

  function playableCells(game: GameState) {
    const cells = new Map<string, BoardCell>()
    for (let q = -boardRadius; q <= boardRadius; q += 1) {
      for (let r = -boardRadius; r <= boardRadius; r += 1) {
        const s = -q - r
        if (Math.max(Math.abs(q), Math.abs(s), Math.abs(r)) <= boardRadius) {
          cells.set(`${q},${s},${r}`, { q, s, r, pieces: [] })
        }
      }
    }
    for (const cell of game.board) {
      cells.set(coordKey(cell), cell)
    }
    for (const action of game.legal_actions) {
      if (action.type === 'place' || action.type === 'move' || action.type === 'pillbug_special') {
        const key = coordKey(action.to)
        if (!cells.has(key)) {
          cells.set(key, { ...action.to, pieces: [] })
        }
      }
    }
    return [...cells.values()].sort((a, b) => a.r - b.r || a.q - b.q || a.s - b.s)
  }

  function boardPoint(position: Position) {
    const size = 38
    return {
      x: size * Math.sqrt(3) * (position.q + position.r / 2),
      y: size * 1.5 * position.r,
    }
  }

  function boardViewBox(cells: BoardCell[]) {
    const points = cells.map(boardPoint)
    const xs = points.map((point) => point.x)
    const ys = points.map((point) => point.y)
    const padding = 90
    const minX = Math.min(...xs) - padding
    const maxX = Math.max(...xs) + padding
    const minY = Math.min(...ys) - padding
    const maxY = Math.max(...ys) + padding
    return `${minX} ${minY} ${maxX - minX} ${maxY - minY}`
  }

  function hexPoints(position: Position) {
    const center = boardPoint(position)
    const radius = 34
    return Array.from({ length: 6 }, (_, index) => {
      const angle = (Math.PI / 180) * (60 * index - 30)
      return `${center.x + radius * Math.cos(angle)},${center.y + radius * Math.sin(angle)}`
    }).join(' ')
  }

  function coordKey(position: Position) {
    return `${position.q},${position.s},${position.r}`
  }

  function statusText(status: string) {
    return status.replaceAll('_', ' ')
  }

  function isFinishedStatus(status: string) {
    return status === 'white_win' || status === 'black_win' || status === 'draw' || status === 'cancelled'
  }

  function resultTitle(game: GameState) {
    if (game.current_status === 'white_win') return 'White wins'
    if (game.current_status === 'black_win') return 'Black wins'
    if (game.current_status === 'draw') return 'Draw'
    return 'Game ended'
  }

  function resultDetail(game: GameState) {
    const moves = game.actions.filter((action) => action.type !== 'cannot_move').length
    const moveLabel = moves === 1 ? 'move' : 'moves'
    if (game.current_status === 'draw') return `Draw after ${moves} ${moveLabel}.`
    if (game.current_status === 'cancelled') return `Cancelled after ${moves} ${moveLabel}.`
    return `${resultTitle(game)} after ${moves} ${moveLabel}.`
  }

  function actionSummary(action: GameAction) {
    if (action.type === 'place') return `${action.turn} placed ${action.piece_type} at ${coordKey(action.to)}`
    if (action.type === 'move') return `${action.turn} moved ${coordKey(action.from)} to ${coordKey(action.to)}`
    if (action.type === 'pillbug_special') {
      return `${action.turn} used pillbug ${coordKey(action.from)} to ${coordKey(action.to)}`
    }
    return `${action.turn} could not move`
  }
</script>

{#if view === 'play' && api.user && currentGame}
  <main class="shell play-shell">
    <header class="topbar">
      <button class="brand" type="button" onclick={leaveGame}>
        <span class="brand-mark">H</span>
        <span>Hive</span>
      </button>

      <div class="session">
        <span>{api.user.username}</span>
        <button class="ghost" type="button" onclick={leaveGame}>Games</button>
        <button class="ghost" type="button" onclick={logout}>Sign out</button>
      </div>
    </header>

    <section class="play-layout">
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
            <strong>{currentGame.viewer_color ?? 'observer'}</strong>
          </div>
          <div>
            <span>Move</span>
            <strong>{currentGame.move_number}</strong>
          </div>
          <div>
            <span>Actions</span>
            <strong>{currentGame.actions.length}</strong>
          </div>
        </div>

        <section class="hand-panel">
          <div class="panel-heading tight">
            <h2>Hand</h2>
            <span>{isMyTurn ? 'Select to place' : 'Waiting'}</span>
          </div>

          <div class="hand-grid">
            {#each pieceTypes as piece}
              <button
                class={`hand-piece hand-${piece}`}
                class:active={selectedHandPiece === piece}
                class:empty={!currentGame.viewer_color || currentGame.inventories[currentGame.viewer_color][piece] === 0}
                type="button"
                onclick={() => selectHandPiece(piece)}
                disabled={!isMyTurn || !currentGame.viewer_color || currentGame.inventories[currentGame.viewer_color][piece] === 0}
                title={piece}
              >
                <span>{pieceLabels[piece]}</span>
                <strong>{currentGame.viewer_color ? currentGame.inventories[currentGame.viewer_color][piece] : 0}</strong>
              </button>
            {/each}
          </div>
        </section>

        <button class="secondary history-button" type="button" onclick={() => (showHistoryModal = true)}>
          Show history
          <span>{currentGame.actions.length}</span>
        </button>

        {#if playError}
          <p class="error">{playError}</p>
        {/if}
      </aside>

      <section class="board-stage" aria-label="Hive board">
        <svg viewBox={viewBox} role="img">
          {#each boardCells as cell}
            {@const point = boardPoint(cell)}
            {@const topPiece = cell.pieces.at(-1)}
            <g
              class:selected={selectedPosition === coordKey(cell)}
              class:highlighted={isHighlighted(cell)}
              class:occupied={cell.pieces.length > 0}
              class="hex-cell"
              role="button"
              tabindex="0"
              onclick={() => clickBoardCell(cell)}
              onkeydown={(event) => {
                if (event.key === 'Enter' || event.key === ' ') {
                  event.preventDefault()
                  clickBoardCell(cell)
                }
              }}
            >
              <polygon points={hexPoints(cell)} />
              {#if topPiece}
                <g class={`piece-token owner-${topPiece.color} piece-${topPiece.piece_type}`} transform={`translate(${point.x} ${point.y})`}>
                  <circle class="piece-disc" r="25" />
                  {#if topPiece.piece_type === 'queen'}
                    <path class="piece-symbol" d="M -13 8 L -10 -8 L -4 0 L 0 -12 L 4 0 L 10 -8 L 13 8 Z" />
                    <circle class="symbol-dot" cx="0" cy="-3" r="3" />
                  {:else if topPiece.piece_type === 'ant'}
                    <circle class="piece-symbol" cx="-9" cy="0" r="5" />
                    <circle class="piece-symbol" cx="0" cy="0" r="6" />
                    <circle class="piece-symbol" cx="10" cy="0" r="5" />
                    <path class="symbol-line" d="M -5 -4 L -14 -12 M -5 4 L -14 12 M 4 -5 L 0 -15 M 4 5 L 0 15 M 10 -4 L 18 -11 M 10 4 L 18 11" />
                  {:else if topPiece.piece_type === 'beetle'}
                    <ellipse class="piece-symbol" cx="0" cy="2" rx="12" ry="14" />
                    <path class="symbol-line" d="M 0 -10 L 0 14 M -9 -2 L 9 -2 M -7 6 L 7 6" />
                  {:else if topPiece.piece_type === 'grasshopper'}
                    <ellipse class="piece-symbol" cx="-3" cy="0" rx="7" ry="12" transform="rotate(-28)" />
                    <path class="symbol-line" d="M 4 -5 L 17 -15 M 5 5 L 18 15 M -7 8 L -17 15 M -4 -8 L -12 -14" />
                  {:else if topPiece.piece_type === 'spider'}
                    <circle class="piece-symbol" cx="0" cy="0" r="10" />
                    <path class="symbol-line" d="M -7 -7 L -18 -16 M -2 -10 L -7 -20 M 2 -10 L 7 -20 M 7 -7 L 18 -16 M -7 7 L -18 16 M -2 10 L -7 20 M 2 10 L 7 20 M 7 7 L 18 16" />
                  {:else if topPiece.piece_type === 'mosquito'}
                    <ellipse class="piece-symbol soft" cx="-7" cy="-3" rx="8" ry="12" transform="rotate(-30)" />
                    <ellipse class="piece-symbol soft" cx="7" cy="-3" rx="8" ry="12" transform="rotate(30)" />
                    <path class="symbol-line" d="M 0 -2 L 0 14 M 0 -8 L 0 -19 M -4 8 L -12 16 M 4 8 L 12 16" />
                  {:else if topPiece.piece_type === 'ladybug'}
                    <circle class="piece-symbol" cx="0" cy="1" r="13" />
                    <path class="symbol-line" d="M 0 -11 L 0 14" />
                    <circle class="symbol-spot" cx="-6" cy="-4" r="2.5" />
                    <circle class="symbol-spot" cx="6" cy="-4" r="2.5" />
                    <circle class="symbol-spot" cx="-5" cy="6" r="2.5" />
                    <circle class="symbol-spot" cx="5" cy="6" r="2.5" />
                  {:else if topPiece.piece_type === 'pillbug'}
                    <ellipse class="piece-symbol" cx="0" cy="0" rx="15" ry="10" />
                    <path class="symbol-line" d="M -9 -8 C -12 -2 -12 2 -9 8 M -3 -10 C -5 -3 -5 3 -3 10 M 3 -10 C 5 -3 5 3 3 10 M 9 -8 C 12 -2 12 2 9 8" />
                  {/if}
                </g>
                {#if cell.pieces.length > 1}
                  <text class="stack-count" x={point.x + 24} y={point.y - 21} text-anchor="middle">
                    {cell.pieces.length}
                  </text>
                {/if}
              {/if}
            </g>
          {/each}
        </svg>
      </section>
    </section>

    {#if showHistoryModal}
      <div class="modal-backdrop" role="presentation" onclick={() => (showHistoryModal = false)}>
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
              <p class="eyebrow">Game #{currentGame.id}</p>
              <h2>History</h2>
            </div>
            <button class="ghost" type="button" onclick={() => (showHistoryModal = false)}>Close</button>
          </div>

          <div class="history-list full">
            {#each currentGame.actions.toReversed() as action}
              <div>
                <strong>#{action.move_number}</strong>
                <span>{actionSummary(action)}</span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    {#if showResultModal && isFinishedStatus(currentGame.current_status)}
      <div class="modal-backdrop" role="presentation" onclick={() => (showResultModal = false)}>
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
          <p>{resultDetail(currentGame)}</p>
          <div class="button-row">
            <button
              class="secondary"
              type="button"
              onclick={() => {
                showResultModal = false
                showHistoryModal = true
              }}
            >
              Show history
            </button>
            <button class="primary" type="button" onclick={() => (showResultModal = false)}>Close</button>
          </div>
        </div>
      </div>
    {/if}
  </main>
{:else if view === 'dashboard' && api.user}
  <main class="shell">
    <header class="topbar">
      <button class="brand" type="button" onclick={() => (view = 'dashboard')}>
        <span class="brand-mark">H</span>
        <span>Hive</span>
      </button>

      <div class="session">
        <span>{api.user.username}</span>
        <button class="ghost" type="button" onclick={logout}>Sign out</button>
      </div>
    </header>

    <section class="workspace">
      <div class="page-heading">
        <p>Multiplayer lobby</p>
        <h1>Invitations</h1>
      </div>

      <div class="dashboard-grid">
        <section class="panel">
          <div class="panel-heading">
            <h2>Create invite</h2>
            <span>{waitingGames.length} waiting</span>
          </div>

          <div class="segmented" aria-label="Creator color">
            <button class:active={creatorColor === 'white'} type="button" onclick={() => (creatorColor = 'white')}>
              White
            </button>
            <button class:active={creatorColor === 'black'} type="button" onclick={() => (creatorColor = 'black')}>
              Black
            </button>
          </div>

          <div class="toggles">
            <label>
              <input type="checkbox" bind:checked={mosquitoEnabled} />
              <span>Mosquito</span>
            </label>
            <label>
              <input type="checkbox" bind:checked={ladybugEnabled} />
              <span>Ladybug</span>
            </label>
            <label>
              <input type="checkbox" bind:checked={pillbugEnabled} />
              <span>Pillbug</span>
            </label>
          </div>

          <button class="primary" type="button" onclick={createGame} disabled={gameBusy}>
            {gameBusy ? 'Creating...' : 'Create game'}
          </button>
        </section>

        <section class="panel">
          <div class="panel-heading">
            <h2>Join invite</h2>
            <span>Code or link</span>
          </div>

          <label class="field">
            <span>Invite code</span>
            <input bind:value={inviteCode} autocomplete="off" placeholder="AB12CD34EF" />
          </label>

          <div class="button-row">
            <button class="secondary" type="button" onclick={previewInvite} disabled={gameBusy || !inviteCode.trim()}>
              Preview
            </button>
            <button class="primary" type="button" onclick={joinInvite} disabled={gameBusy || !inviteCode.trim()}>
              Join
            </button>
          </div>

          {#if selectedInvite}
            <div class="invite-preview">
              <span>Game #{selectedInvite.id}</span>
              <strong>{selectedInvite.white_user_id ? 'Black seat open' : 'White seat open'}</strong>
            </div>
          {/if}
        </section>
      </div>

      {#if gameError}
        <p class="error">{gameError}</p>
      {/if}

      <section class="games-section">
        <div class="section-heading">
          <h2>Your games</h2>
          <span>{gameLoadBusy ? 'Loading' : `${games.length} total`}</span>
        </div>

        {#if games.length === 0}
          <div class="empty-state">
            <h3>No games yet</h3>
            <p>Create an invite or join one with a code.</p>
          </div>
        {:else}
          <div class="game-list">
            {#each games as game}
              <article class="game-card">
                <div>
                  <h3>Game #{game.id}</h3>
                  <p>{statusText(game.current_status)}</p>
                </div>
                <div class="game-actions">
                  {#if game.invite_code}
                    <code>{game.invite_code}</code>
                  {/if}
                  {#if game.current_status === 'in_progress'}
                    <button class="primary small" type="button" onclick={() => openGame(game.id)}>Play</button>
                  {:else}
                    <span class="pill">{statusText(game.current_status)}</span>
                  {/if}
                </div>
              </article>
            {/each}
          </div>
        {/if}
      </section>
    </section>
  </main>
{:else}
  <main class="public-shell">
    <header class="topbar public">
      <button class="brand" type="button" onclick={() => (view = 'landing')}>
        <span class="brand-mark">H</span>
        <span>Hive</span>
      </button>
      <nav>
        <button class="ghost" type="button" onclick={() => showAuth('login')}>Login</button>
        <button class="primary small" type="button" onclick={() => showAuth('register')}>Register</button>
      </nav>
    </header>

    {#if view === 'landing'}
      <section class="hero">
        <div class="hero-copy">
          <p>Two-player strategy</p>
          <h1>Hive matches with private invitations.</h1>
          <div class="hero-actions">
            <button class="primary" type="button" onclick={() => showAuth('register')}>Create account</button>
            <button class="secondary" type="button" onclick={() => showAuth('login')}>Login</button>
          </div>
        </div>
        <div class="board-preview" aria-hidden="true">
          {#each Array(19) as _, index}
            <span class:dark={index % 3 === 0} class:gold={index % 4 === 0}></span>
          {/each}
        </div>
      </section>
    {:else}
      <section class="auth-view">
        <form class="auth-panel" onsubmit={(event) => { event.preventDefault(); submitAuth() }}>
          <div>
            <p>{authMode === 'login' ? 'Welcome back' : 'New player'}</p>
            <h1>{authMode === 'login' ? 'Login' : 'Register'}</h1>
          </div>

          {#if authMode === 'register'}
            <label class="field">
              <span>Username</span>
              <input bind:value={username} autocomplete="username" required />
            </label>
          {/if}

          <label class="field">
            <span>Email</span>
            <input bind:value={email} type="email" autocomplete="email" required />
          </label>

          <label class="field">
            <span>Password</span>
            <input bind:value={password} type="password" autocomplete={authMode === 'login' ? 'current-password' : 'new-password'} required />
          </label>

          {#if authError}
            <p class="error">{authError}</p>
          {/if}

          <button class="primary" type="submit" disabled={authBusy}>
            {authBusy ? 'Working...' : authMode === 'login' ? 'Login' : 'Create account'}
          </button>

          <button
            class="text-button"
            type="button"
            onclick={() => showAuth(authMode === 'login' ? 'register' : 'login')}
          >
            {authMode === 'login' ? 'Need an account?' : 'Already registered?'}
          </button>
        </form>
      </section>
    {/if}
  </main>
{/if}
