import type { GameAction, GameState, PieceType, Position } from './api.svelte'

export const playerColors = ['white', 'black'] as const
export const pieceTypes: PieceType[] = [
  'queen',
  'ant',
  'beetle',
  'grasshopper',
  'spider',
  'mosquito',
  'ladybug',
  'pillbug',
]

export const pieceAssets: Record<PieceType, string> = {
  queen: '/Queen.svg',
  ant: '/Ant.svg',
  beetle: '/Beetle.svg',
  grasshopper: '/Grasshopper.svg',
  spider: '/Spider.svg',
  mosquito: '/Mosquito.svg',
  ladybug: '/Ladybug.svg',
  pillbug: '/Pillbug.svg',
}

export function readableError(error: unknown) {
  return error instanceof Error ? error.message : 'Something went wrong'
}

export function coordKey(position: Position) {
  return `${position.q},${position.s},${position.r}`
}

export function statusText(status: string) {
  return status.replaceAll('_', ' ')
}

export function isFinishedStatus(status: string) {
  return status === 'white_win' || status === 'black_win' || status === 'draw' || status === 'cancelled'
}

export function resultTitle(game: GameState) {
  if (game.current_status === 'white_win') return 'White wins'
  if (game.current_status === 'black_win') return 'Black wins'
  if (game.current_status === 'draw') return 'Draw'
  return 'Game ended'
}

export function actionSummary(action: GameAction) {
  if (action.type === 'place') return `${action.turn} placed ${action.piece_type} at ${coordKey(action.to)}`
  if (action.type === 'move') return `${action.turn} moved ${coordKey(action.from)} to ${coordKey(action.to)}`
  if (action.type === 'pillbug_special') {
    return `${action.turn} used pillbug ${coordKey(action.from)} to ${coordKey(action.to)}`
  }
  return `${action.turn} could not move`
}

export function isSoundAction(action: GameAction) {
  return action.type === 'place' || action.type === 'move' || action.type === 'pillbug_special'
}
