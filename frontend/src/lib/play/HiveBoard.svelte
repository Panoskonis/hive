<script lang="ts">
  import type { BoardCell } from '$lib/api.svelte'
  import { boardPoint, coordinateLabels, hexPoints } from '$lib/board-geometry'
  import { coordKey, pieceAssets } from '$lib/hive-ui'

  let {
    boardCells,
    viewBox,
    selectedPosition,
    showCoordinates,
    lastMoveFromKey,
    lastMoveToKey,
    isBoardPanning,
    isHighlighted,
    clickBoardCell,
    handleBoardWheel,
    startBoardPan,
    moveBoardPan,
    stopBoardPan,
    zoomBoard,
    resetBoardCamera,
  }: {
    boardCells: BoardCell[]
    viewBox: string
    selectedPosition: string | null
    showCoordinates: boolean
    lastMoveFromKey: string | null
    lastMoveToKey: string | null
    isBoardPanning: boolean
    isHighlighted: (cell: BoardCell) => boolean
    clickBoardCell: (cell: BoardCell) => void | Promise<void>
    handleBoardWheel: (event: WheelEvent) => void
    startBoardPan: (event: PointerEvent) => void
    moveBoardPan: (event: PointerEvent) => void
    stopBoardPan: (event: PointerEvent) => void
    zoomBoard: (delta: number) => void
    resetBoardCamera: () => void
  } = $props()
</script>

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

  <svg viewBox={viewBox} role="img">
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
