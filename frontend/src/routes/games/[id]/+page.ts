export function load({ params }) {
  return {
    gameId: Number(params.id),
    gameLabel: params.id,
  }
}
