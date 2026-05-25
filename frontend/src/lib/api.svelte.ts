export type User = {
  id: number
  username: string
  email: string
  role: string
}

export type AuthResponse = {
  token: string
  expires_at: string
  user: User
}

export type Game = {
  id: number
  creator_user_id: number
  white_user_id: number | null
  black_user_id: number | null
  invite_code: string | null
  created_at: string
  started_at: string | null
  ended_at: string | null
  current_status: string
  mosquito_enabled: boolean
  ladybug_enabled: boolean
  pillbug_enabled: boolean
}

export type PlayerColor = 'white' | 'black'
export type PieceType =
  | 'queen'
  | 'ant'
  | 'beetle'
  | 'grasshopper'
  | 'spider'
  | 'mosquito'
  | 'ladybug'
  | 'pillbug'

export type Position = {
  q: number
  s: number
  r: number
}

export type Piece = {
  color: PlayerColor
  piece_type: PieceType
}

export type BoardCell = Position & {
  pieces: Piece[]
}

export type Inventory = Record<PieceType, number>

export type GameAction =
  | { id?: number | null; move_number: number; turn: PlayerColor; type: 'place'; piece_type: PieceType; to: Position }
  | { id?: number | null; move_number: number; turn: PlayerColor; type: 'move'; from: Position; to: Position }
  | { id?: number | null; move_number: number; turn: PlayerColor; type: 'pillbug_special'; from: Position; to: Position }
  | { id?: number | null; move_number: number; turn: PlayerColor; type: 'cannot_move' }

export type GameState = Game & {
  viewer_color: PlayerColor | null
  current_turn: PlayerColor
  move_number: number
  board: BoardCell[]
  inventories: {
    white: Inventory
    black: Inventory
  }
  legal_actions: GameAction[]
}

export type CreateGamePayload = {
  creator_color: PlayerColor
  mosquito_enabled: boolean
  ladybug_enabled: boolean
  pillbug_enabled: boolean
}

const apiBase = import.meta.env.VITE_API_BASE_URL ?? '/api'

export class ApiClient {
  token = $state<string | null>(localStorage.getItem('hive.token'))
  user = $state<User | null>(readStoredUser())

  get isAuthenticated() {
    return this.token !== null && this.user !== null
  }

  async register(username: string, email: string, password: string) {
    const auth = await this.request<AuthResponse>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ username, email, password }),
    })
    this.storeAuth(auth)
  }

  async login(email: string, password: string) {
    const auth = await this.request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    })
    this.storeAuth(auth)
  }

  async logout() {
    try {
      await this.request<void>('/auth/logout', {
        method: 'POST',
        body: JSON.stringify({}),
      })
    } finally {
      this.clearAuth()
    }
  }

  async createGame(payload: CreateGamePayload) {
    return this.request<Game>('/games', {
      method: 'POST',
      body: JSON.stringify(payload),
    })
  }

  async joinGame(inviteCode: string) {
    return this.request<Game>('/games/join', {
      method: 'POST',
      body: JSON.stringify({ invite_code: inviteCode }),
    })
  }

  async previewInvite(inviteCode: string) {
    return this.request<Game>(`/games/invites/${encodeURIComponent(inviteCode)}`)
  }

  async listGames() {
    return this.request<Game[]>('/games')
  }

  async getGameState(gameId: number) {
    return this.request<GameState>(`/games/${gameId}/state`)
  }

  async getGameActions(gameId: number) {
    return this.request<GameAction[]>(`/games/${gameId}/actions`)
  }

  async submitAction(gameId: number, action: GameAction) {
    return this.request<GameState>(`/games/${gameId}/actions`, {
      method: 'POST',
      body: JSON.stringify(action),
    })
  }

  private storeAuth(auth: AuthResponse) {
    this.token = auth.token
    this.user = auth.user
    localStorage.setItem('hive.token', auth.token)
    localStorage.setItem('hive.user', JSON.stringify(auth.user))
  }

  private clearAuth() {
    this.token = null
    this.user = null
    localStorage.removeItem('hive.token')
    localStorage.removeItem('hive.user')
  }

  private async request<T>(path: string, init: RequestInit = {}): Promise<T> {
    const headers = new Headers(init.headers)
    headers.set('content-type', 'application/json')

    if (this.token) {
      headers.set('authorization', `Bearer ${this.token}`)
    }

    const response = await fetch(`${apiBase}${path}`, {
      ...init,
      headers,
    })

    if (response.status === 204) {
      return undefined as T
    }

    const data = await response.json().catch(() => null)
    if (!response.ok) {
      throw new Error(data?.error ?? 'Request failed')
    }

    return data as T
  }
}

function readStoredUser() {
  const rawUser = localStorage.getItem('hive.user')
  if (!rawUser) {
    return null
  }

  try {
    return JSON.parse(rawUser) as User
  } catch {
    localStorage.removeItem('hive.user')
    return null
  }
}
