<script lang="ts">
  import { onMount } from 'svelte'
  import {
    ApiClient,
    type BoardCell,
    type Game,
    type GameAction,
    type GameState,
    type PlayerColor,
    type PieceType,
    type Position,
  } from './lib/api.svelte'

  const api = new ApiClient()
  const boardRadius = 12
  const hexSpacingRadius = 38
  const hexDrawRadius = 34
  const hexOrientation = Math.PI / 3
  const coordinateLabelInset = hexDrawRadius * 0.58
  const minBoardZoom = 0.65
  const maxBoardZoom = 3.2
  const playerColors: PlayerColor[] = ['white', 'black']
  const pieceTypes: PieceType[] = ['queen', 'ant', 'beetle', 'grasshopper', 'spider', 'mosquito', 'ladybug', 'pillbug']
  const pieceAssets: Record<PieceType, string> = {
    queen: '/Queen.svg',
    ant: '/Ant.svg',
    beetle: '/Beetle.svg',
    grasshopper: '/Grasshopper.svg',
    spider: '/Spider.svg',
    mosquito: '/Mosquito.svg',
    ladybug: '/Ladybug.svg',
    pillbug: '/Pillbug.svg',
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
  let createMode = $state<'invite' | 'solo'>('invite')
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
  let actionHistory = $state<GameAction[]>([])
  let playError = $state('')
  let selectedHandPiece = $state<PieceType | null>(null)
  let selectedPosition = $state<string | null>(null)
  let showCoordinates = $state(false)
  let showHistoryModal = $state(false)
  let showResultModal = $state(false)
  let soundMuted = $state(localStorage.getItem('hive.soundMuted') === 'true')
  let boardZoom = $state(1.45)
  let boardPanX = $state(0)
  let boardPanY = $state(0)
  let isBoardPanning = $state(false)
  let boardSuppressClick = $state(false)
  let boardPanStart = $state<{ x: number; y: number; panX: number; panY: number } | null>(null)
  let pollTimer: number | null = null
  let lobbyPollTimer: number | null = null

  const waitingGames = $derived(games.filter((game) => game.current_status === 'waiting_for_opponent'))
  const activeGames = $derived(games.filter((game) => game.current_status === 'in_progress'))
  const isSelfPlay = $derived(
    currentGame !== null && currentGame.white_user_id !== null && currentGame.white_user_id === currentGame.black_user_id,
  )
  const activePlayerColor = $derived(
    currentGame ? (isSelfPlay ? currentGame.current_turn : currentGame.viewer_color) : null,
  )
  const isMyTurn = $derived(currentGame !== null && activePlayerColor === currentGame.current_turn)
  const boardCells = $derived(currentGame ? playableCells(currentGame) : [])
  const baseBoardViewBox = $derived(boardViewBox(boardCells))
  const viewBox = $derived(cameraViewBox(baseBoardViewBox, boardZoom, boardPanX, boardPanY))
  const lastPieceAction = $derived(
    [...actionHistory]
      .reverse()
      .find((action) => action.type === 'place' || action.type === 'move' || action.type === 'pillbug_special') ?? null,
  )
  const lastMoveFromKey = $derived(
    lastPieceAction && (lastPieceAction.type === 'move' || lastPieceAction.type === 'pillbug_special')
      ? coordKey(lastPieceAction.from)
      : null,
  )
  const lastMoveToKey = $derived(
    lastPieceAction && (lastPieceAction.type === 'place' || lastPieceAction.type === 'move' || lastPieceAction.type === 'pillbug_special')
      ? coordKey(lastPieceAction.to)
      : null,
  )

  onMount(() => {
    if (api.isAuthenticated) {
      void refreshGames()
    }

    return () => {
      stopPolling()
      stopLobbyPolling()
    }
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
    stopLobbyPolling()
    await api.logout()
    games = []
    selectedInvite = null
    currentGame = null
    actionHistory = []
    showHistoryModal = false
    showResultModal = false
    view = 'landing'
  }

  async function refreshGames(silent = false) {
    if (!api.isAuthenticated) return
    const previousWaitingIds = new Set(
      games.filter((game) => game.current_status === 'waiting_for_opponent').map((game) => game.id),
    )
    if (!silent) gameLoadBusy = true
    gameError = ''

    try {
      const nextGames = await api.listGames()
      const acceptedGame = nextGames.find(
        (game) => previousWaitingIds.has(game.id) && game.current_status === 'in_progress',
      )
      games = nextGames
      if (acceptedGame) {
        inviteCode = ''
        selectedInvite = null
      }
      syncLobbyPolling()
    } catch (error) {
      gameError = readableError(error)
    } finally {
      if (!silent) gameLoadBusy = false
    }
  }

  async function createGame() {
    gameBusy = true
    gameError = ''

    try {
      const game = await api.createGame({
        creator_color: creatorColor,
        self_play: createMode === 'solo',
        mosquito_enabled: mosquitoEnabled,
        ladybug_enabled: ladybugEnabled,
        pillbug_enabled: pillbugEnabled,
      })
      await refreshGames()
      if (createMode === 'solo') {
        inviteCode = ''
        await openGame(game.id)
      } else {
        inviteCode = game.invite_code ?? ''
      }
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
    stopLobbyPolling()
    playError = ''
    selectedHandPiece = null
    selectedPosition = null
    resetBoardCamera()
    actionHistory = []
    showHistoryModal = false
    showResultModal = false

    try {
      const [nextGame, history] = await Promise.all([api.getGameState(gameId), api.getGameActions(gameId)])
      actionHistory = history
      setCurrentGame(nextGame)
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
    actionHistory = []
    selectedHandPiece = null
    selectedPosition = null
    resetBoardCamera()
    showHistoryModal = false
    showResultModal = false
    view = 'dashboard'
    void refreshGames()
  }

  function syncLobbyPolling() {
    if (
      api.isAuthenticated &&
      view === 'dashboard' &&
      games.some((game) => game.current_status === 'waiting_for_opponent')
    ) {
      startLobbyPolling()
    } else {
      stopLobbyPolling()
    }
  }

  function startLobbyPolling() {
    if (lobbyPollTimer !== null) return
    lobbyPollTimer = window.setInterval(() => {
      if (view !== 'dashboard' || !games.some((game) => game.current_status === 'waiting_for_opponent')) {
        stopLobbyPolling()
        return
      }
      void refreshGames(true)
    }, 2000)
  }

  function stopLobbyPolling() {
    if (lobbyPollTimer !== null) {
      window.clearInterval(lobbyPollTimer)
      lobbyPollTimer = null
    }
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
        const previousGame = currentGame
        setCurrentGame(nextGame)
        if (hasGameAdvanced(previousGame, nextGame)) {
          const previousActionCount = actionHistory.length
          const nextHistory = await refreshActionHistory(nextGame.id)
          const newestAction = nextHistory.at(-1)
          if (nextHistory.length > previousActionCount && newestAction && isSoundAction(newestAction)) {
            playMoveSound()
          }
        }
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

  function selectHandPiece(piece: PieceType, color: PlayerColor) {
    if (!canUseInventory(color) || !currentGame || currentGame.inventories[color][piece] <= 0) return
    selectedHandPiece = selectedHandPiece === piece ? null : piece
    selectedPosition = null
  }

  function canUseInventory(color: PlayerColor) {
    return currentGame !== null && isMyTurn && activePlayerColor === color
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

  function resetBoardCamera() {
    boardZoom = 1.45
    boardPanX = 0
    boardPanY = 0
    isBoardPanning = false
    boardSuppressClick = false
    boardPanStart = null
  }

  function zoomBoard(delta: number) {
    boardZoom = clamp(boardZoom + delta, minBoardZoom, maxBoardZoom)
  }

  function handleBoardWheel(event: WheelEvent) {
    event.preventDefault()
    zoomBoard(event.deltaY > 0 ? -0.14 : 0.14)
  }

  function startBoardPan(event: PointerEvent) {
    if (event.button !== 0) return
    if (event.target instanceof Element && event.target.closest('.board-controls')) return
    isBoardPanning = true
    boardSuppressClick = false
    boardPanStart = { x: event.clientX, y: event.clientY, panX: boardPanX, panY: boardPanY }
  }

  function moveBoardPan(event: PointerEvent) {
    if (!boardPanStart) return
    const rect = event.currentTarget instanceof Element ? event.currentTarget.getBoundingClientRect() : null
    if (!rect) return
    const startView = cameraViewBox(baseBoardViewBox, boardZoom, boardPanStart.panX, boardPanStart.panY)
    const deltaX = event.clientX - boardPanStart.x
    const deltaY = event.clientY - boardPanStart.y
    if (Math.hypot(deltaX, deltaY) > 6) {
      boardSuppressClick = true
    }
    boardPanX = boardPanStart.panX - (deltaX / rect.width) * startView.width
    boardPanY = boardPanStart.panY - (deltaY / rect.height) * startView.height
  }

  function stopBoardPan(event: PointerEvent) {
    isBoardPanning = false
    boardPanStart = null
    if (boardSuppressClick) {
      window.setTimeout(() => {
        boardSuppressClick = false
      }, 0)
    }
  }

  async function clickBoardCell(cell: BoardCell) {
    if (boardSuppressClick) {
      boardSuppressClick = false
      return
    }
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
      const nextGame = await api.submitAction(currentGame.id, action)
      appendHistoryAction(action)
      if (isSoundAction(action)) {
        playMoveSound()
      }
      setCurrentGame(nextGame)
      if (nextGame.current_turn === action.turn) {
        await refreshActionHistory(nextGame.id)
      }
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

  function hasGameAdvanced(previousGame: GameState, nextGame: GameState) {
    return (
      previousGame.move_number !== nextGame.move_number ||
      previousGame.current_turn !== nextGame.current_turn ||
      previousGame.current_status !== nextGame.current_status
    )
  }

  function appendHistoryAction(action: GameAction) {
    if (
      action.id !== null &&
      action.id !== undefined &&
      actionHistory.some((historyAction) => historyAction.id === action.id)
    ) {
      return
    }
    actionHistory = [...actionHistory, { ...action, id: action.id ?? null }]
  }

  async function refreshActionHistory(gameId: number) {
    const nextHistory = await api.getGameActions(gameId)
    actionHistory = nextHistory
    return nextHistory
  }

  function isSoundAction(action: GameAction) {
    return action.type === 'place' || action.type === 'move' || action.type === 'pillbug_special'
  }

  function toggleSoundMuted() {
    soundMuted = !soundMuted
    localStorage.setItem('hive.soundMuted', String(soundMuted))
  }

  function playMoveSound() {
    if (soundMuted) return
    const AudioContextConstructor = window.AudioContext
    if (!AudioContextConstructor) return

    const context = new AudioContextConstructor()
    const now = context.currentTime
    const output = context.createGain()
    output.gain.setValueAtTime(0.0001, now)
    output.gain.exponentialRampToValueAtTime(0.18, now + 0.006)
    output.gain.exponentialRampToValueAtTime(0.0001, now + 0.13)
    output.connect(context.destination)

    for (const [index, frequency] of [720, 410].entries()) {
      const oscillator = context.createOscillator()
      oscillator.type = 'triangle'
      oscillator.frequency.setValueAtTime(frequency, now + index * 0.035)
      oscillator.connect(output)
      oscillator.start(now + index * 0.035)
      oscillator.stop(now + 0.115 + index * 0.035)
    }

    window.setTimeout(() => {
      void context.close()
    }, 220)
  }

  async function openHistory() {
    if (currentGame) {
      try {
        await refreshActionHistory(currentGame.id)
      } catch (error) {
        playError = readableError(error)
      }
    }
    showHistoryModal = true
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
    const cos60 = Math.cos(Math.PI / 3)
    const sin60 = Math.sin(Math.PI / 3)
    return {
      x: hexSpacingRadius * (position.q - cos60 * (position.r + position.s)),
      y: hexSpacingRadius * sin60 * (position.s - position.r),
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
    return { x: minX, y: minY, width: maxX - minX, height: maxY - minY }
  }

  function cameraViewBox(
    base: { x: number; y: number; width: number; height: number },
    zoom: number,
    panX: number,
    panY: number,
  ) {
    const width = base.width / zoom
    const height = base.height / zoom
    const x = base.x + (base.width - width) / 2 + panX
    const y = base.y + (base.height - height) / 2 + panY
    return { x, y, width, height, value: `${x} ${y} ${width} ${height}` }
  }

  function clamp(value: number, min: number, max: number) {
    return Math.min(max, Math.max(min, value))
  }

  function hexPoints(position: Position) {
    const center = boardPoint(position)
    return Array.from({ length: 6 }, (_, index) => {
      const angle = hexOrientation + index * (Math.PI / 3)
      return `${center.x + hexDrawRadius * Math.cos(angle)},${center.y + hexDrawRadius * Math.sin(angle)}`
    }).join(' ')
  }

  function coordinateLabels(position: Position) {
    const center = boardPoint(position)
    return [
      { value: position.q, angleIndex: 0 },
      { value: position.r, angleIndex: 4 },
      { value: position.s, angleIndex: 2 },
    ].map((label) => {
      const angle = hexOrientation + label.angleIndex * (Math.PI / 3)
      return {
        value: label.value,
        x: center.x + coordinateLabelInset * Math.cos(angle),
        y: center.y + coordinateLabelInset * Math.sin(angle),
      }
    })
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
    const moves = actionHistory.filter((action) => action.type !== 'cannot_move').length
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
                  <span>{currentGame.current_turn === color ? 'turn' : 'view'}</span>
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
          onclick={() => (showCoordinates = !showCoordinates)}
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

      <section
        class="board-stage"
        class:panning={isBoardPanning}
        aria-label="Hive board"
        onwheel={handleBoardWheel}
        onpointerdown={startBoardPan}
        onpointermove={moveBoardPan}
        onpointerup={stopBoardPan}
        onpointercancel={stopBoardPan}
      >
        <div class="board-controls" aria-label="Board zoom controls">
          <button type="button" onclick={() => zoomBoard(0.18)} aria-label="Zoom in">+</button>
          <button type="button" onclick={() => zoomBoard(-0.18)} aria-label="Zoom out">-</button>
          <button type="button" onclick={resetBoardCamera} aria-label="Reset board view">Reset</button>
        </div>

        <svg viewBox={viewBox.value} role="img">
          {#each boardCells as cell}
            {@const point = boardPoint(cell)}
            {@const topPiece = cell.pieces.at(-1)}
            {@const key = coordKey(cell)}
            <g
              class:selected={selectedPosition === key}
              class:highlighted={isHighlighted(cell)}
              class:last-move-from={lastMoveFromKey === key}
              class:last-move-to={lastMoveToKey === key}
              class:occupied={cell.pieces.length > 0}
              class:owner-white={topPiece?.color === 'white'}
              class:owner-black={topPiece?.color === 'black'}
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
                <g
                  class={`piece-token owner-${topPiece.color} piece-${topPiece.piece_type}`}
                  class:last-moved-piece={lastMoveToKey === key}
                  transform={`translate(${point.x} ${point.y})`}
                >
                  <image href={pieceAssets[topPiece.piece_type]} x="-22" y="-25" width="44" height="50" />
                </g>
                {#if cell.pieces.length > 1}
                  <text class="stack-count" x={point.x + 24} y={point.y - 21} text-anchor="middle">
                    {cell.pieces.length}
                  </text>
                {/if}
              {/if}
              {#if showCoordinates}
                <g class={`coordinate-labels ${topPiece?.color === 'black' ? 'on-black' : ''}`}>
                  {#each coordinateLabels(cell) as label}
                    <text x={label.x} y={label.y} text-anchor="middle" dominant-baseline="central">
                      {label.value}
                    </text>
                  {/each}
                </g>
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
                void openHistory()
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
        <h1>Games</h1>
      </div>

      <div class="dashboard-grid">
        <section class="panel">
          <div class="panel-heading">
            <h2>{createMode === 'invite' ? 'Create invite' : 'Create solo game'}</h2>
            <span>{createMode === 'invite' ? `${waitingGames.length} waiting` : `${activeGames.length} active`}</span>
          </div>

          <div class="segmented" aria-label="Game creation mode">
            <button class:active={createMode === 'invite'} type="button" onclick={() => (createMode = 'invite')}>
              Invite
            </button>
            <button class:active={createMode === 'solo'} type="button" onclick={() => (createMode = 'solo')}>
              Solo
            </button>
          </div>

          {#if createMode === 'invite'}
            <div class="segmented" aria-label="Creator color">
              <button class:active={creatorColor === 'white'} type="button" onclick={() => (creatorColor = 'white')}>
                White
              </button>
              <button class:active={creatorColor === 'black'} type="button" onclick={() => (creatorColor = 'black')}>
                Black
              </button>
            </div>
          {/if}

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
            {gameBusy ? 'Creating...' : createMode === 'invite' ? 'Create invite' : 'Start solo game'}
          </button>

          {#if createMode === 'invite' && waitingGames.length > 0}
            <p class="lobby-status">Checking pending invites...</p>
          {/if}
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
          <h1>HIVE Board Game</h1>
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
