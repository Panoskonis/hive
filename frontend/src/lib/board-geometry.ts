import type { BoardCell, GameState, Position } from './api.svelte'
import { coordKey } from './hive-ui'

const boardRadius = 12
const hexSpacingRadius = 38
export const hexDrawRadius = 34
export const minBoardZoom = 0.65
export const maxBoardZoom = 3.2
const hexOrientation = Math.PI / 3
const coordinateLabelInset = hexDrawRadius * 0.58

export function playableCells(game: GameState) {
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

export function boardPoint(position: Position) {
  const cos60 = Math.cos(Math.PI / 3)
  const sin60 = Math.sin(Math.PI / 3)
  return {
    x: hexSpacingRadius * (position.q - cos60 * (position.r + position.s)),
    y: hexSpacingRadius * sin60 * (position.s - position.r),
  }
}

export function boardViewBox(cells: BoardCell[]) {
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

export function cameraViewBox(
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

export function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value))
}

export function hexPoints(position: Position) {
  const center = boardPoint(position)
  return Array.from({ length: 6 }, (_, index) => {
    const angle = hexOrientation + index * (Math.PI / 3)
    return `${center.x + hexDrawRadius * Math.cos(angle)},${center.y + hexDrawRadius * Math.sin(angle)}`
  }).join(' ')
}

export function coordinateLabels(position: Position) {
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
