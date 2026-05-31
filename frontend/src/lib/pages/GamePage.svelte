<script lang="ts">
  import { goto } from '$app/navigation'
  import { onMount } from 'svelte'
  import Topbar from '$lib/components/Topbar.svelte'
  import GameSidebar from '$lib/play/GameSidebar.svelte'
  import HiveBoard from '$lib/play/HiveBoard.svelte'
  import HistoryModal from '$lib/play/HistoryModal.svelte'
  import ResultModal from '$lib/play/ResultModal.svelte'
  import type { BoardCell, GameAction, GameState, PlayerColor, PieceType } from '$lib/api.svelte'
  import {
    cameraViewBox,
    clamp,
    maxBoardZoom,
    minBoardZoom,
    playableCells,
    boardViewBox,
  } from '$lib/board-geometry'
  import { coordKey, isFinishedStatus, isSoundAction, readableError } from '$lib/hive-ui'
  import { playMoveSound } from '$lib/move-sound'
  import { api, signOut } from '$lib/session.svelte'

  let { gameId }: { gameId: number } = $props()

  let currentGame = $state<GameState | null>(null)
  let actionHistory = $state<GameAction[]>([])
  let playError = $state('')
  let selectedHandPiece = $state<PieceType | null>(null)
  let selectedPosition = $state<string | null>(null)
  let showCoordinates = $state(false)
  let showHistoryModal = $state(false)
  let showResultModal = $state(false)
  let soundMuted = $state(false)
  let boardZoom = $state(1.45)
  let boardPanX = $state(0)
  let boardPanY = $state(0)
  let isBoardPanning = $state(false)
  let boardSuppressClick = $state(false)
  let boardPanStart = $state<{ x: number; y: number; panX: number; panY: number } | null>(null)
  let pollTimer: number | null = null

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
    soundMuted = localStorage.getItem('hive.soundMuted') === 'true'

    if (!api.isAuthenticated) {
      void goto('/login')
      return
    }

    void loadGame()

    return () => {
      stopPolling()
    }
  })

  async function loadGame() {
    stopPolling()
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
      startPolling()
    } catch (error) {
      playError = readableError(error)
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
    void goto('/dashboard')
  }

  function startPolling() {
    stopPolling()
    if (!currentGame || !shouldPollGame(currentGame)) return
    pollTimer = window.setInterval(async () => {
      if (!currentGame || !shouldPollGame(currentGame)) {
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
            playMoveSound(soundMuted)
          }
        }
        if (!shouldPollGame(nextGame)) {
          stopPolling()
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

  function shouldPollGame(game: GameState) {
    return game.current_status === 'waiting_for_opponent' || game.current_status === 'in_progress'
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

  function stopBoardPan() {
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
        playMoveSound(soundMuted)
      }
      setCurrentGame(nextGame)
      if (nextGame.current_turn === action.turn) {
        await refreshActionHistory(nextGame.id)
      }
      selectedHandPiece = null
      selectedPosition = null
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

  async function refreshActionHistory(nextGameId: number) {
    const nextHistory = await api.getGameActions(nextGameId)
    actionHistory = nextHistory
    return nextHistory
  }

  function toggleSoundMuted() {
    soundMuted = !soundMuted
    localStorage.setItem('hive.soundMuted', String(soundMuted))
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
</script>

{#if api.user && currentGame}
  <main class="shell play-shell">
    <Topbar
      userName={api.user.username}
      onBrand={leaveGame}
      onSignOut={signOut}
      showGames
      onGames={leaveGame}
    />

    <section class="play-layout">
      <GameSidebar
        {currentGame}
        {actionHistory}
        {isSelfPlay}
        {isMyTurn}
        {activePlayerColor}
        {selectedHandPiece}
        {showCoordinates}
        {soundMuted}
        {playError}
        {canUseInventory}
        {selectHandPiece}
        {openHistory}
        toggleCoordinates={() => (showCoordinates = !showCoordinates)}
        {toggleSoundMuted}
      />

      <HiveBoard
        {boardCells}
        viewBox={viewBox.value}
        {selectedPosition}
        {showCoordinates}
        {lastMoveFromKey}
        {lastMoveToKey}
        {isBoardPanning}
        {isHighlighted}
        {clickBoardCell}
        {handleBoardWheel}
        {startBoardPan}
        {moveBoardPan}
        {stopBoardPan}
        {zoomBoard}
        {resetBoardCamera}
      />
    </section>

    {#if showHistoryModal}
      <HistoryModal gameId={currentGame.id} {actionHistory} onClose={() => (showHistoryModal = false)} />
    {/if}

    {#if showResultModal && isFinishedStatus(currentGame.current_status)}
      <ResultModal
        {currentGame}
        {actionHistory}
        onClose={() => (showResultModal = false)}
        {openHistory}
      />
    {/if}
  </main>
{/if}
