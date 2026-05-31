<script lang="ts">
  import { goto } from '$app/navigation'
  import { onMount } from 'svelte'
  import Topbar from '$lib/components/Topbar.svelte'
  import type { Game, PlayerColor } from '$lib/api.svelte'
  import { readableError, statusText } from '$lib/hive-ui'
  import { api, signOut } from '$lib/session.svelte'

  let creatorColor = $state<PlayerColor>('white')
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
  let lobbyPollTimer: number | null = null

  const waitingGames = $derived(games.filter((game) => game.current_status === 'waiting_for_opponent'))
  const activeGames = $derived(games.filter((game) => game.current_status === 'in_progress'))

  onMount(() => {
    if (!api.isAuthenticated) {
      void goto('/login')
      return
    }

    void refreshGames()

    return () => {
      stopLobbyPolling()
    }
  })

  async function refreshGames(silent = false) {
    if (!api.isAuthenticated) return
    const previousWaitingIds = new Set(
      games.filter((game) => game.current_status === 'waiting_for_opponent').map((game) => game.id),
    )
    if (!silent) gameLoadBusy = true
    gameError = ''

    try {
      const nextGames = await api.listGames().then((games) => games.sort((a, b) => b.id - a.id))
      const acceptedGame = nextGames.find(
        (game) => previousWaitingIds.has(game.id) && game.current_status === 'in_progress',
      )
      games = nextGames
      if (acceptedGame) {
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
      } else {
        inviteCode = game.invite_code ?? ''
      }
      await goto(`/games/${game.id}`)

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
      await goto(`/games/${game.id}`)
    } catch (error) {
      gameError = readableError(error)
    } finally {
      gameBusy = false
    }
  }

  function syncLobbyPolling() {
    if (api.isAuthenticated && games.some((game) => game.current_status === 'waiting_for_opponent')) {
      startLobbyPolling()
    } else {
      stopLobbyPolling()
    }
  }

  function startLobbyPolling() {
    if (lobbyPollTimer !== null) return
    lobbyPollTimer = window.setInterval(() => {
      if (!games.some((game) => game.current_status === 'waiting_for_opponent')) {
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
</script>

{#if api.user}
  <main class="shell">
    <Topbar userName={api.user.username} onBrand={() => goto('/dashboard')} onSignOut={signOut} />

    <section class="workspace">
      <div class="dashboard-grid">
        <section class="panel">
          <div class="panel-heading">
            <h2>{createMode === 'invite' ? 'Create multiplayer game' : 'Create solo game'}</h2>
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
            Create Game
          </button>
        </section>

        <section class="panel">
          <div class="panel-heading">
            <h2>Join game</h2>
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
                </div>
                <div class="game-actions">
                  {#if game.invite_code}
                    <code>Invite code: {game.invite_code}</code>
                  {/if}
                  <span class="pill">{statusText(game.current_status)}</span>
                  {#if game.current_status === 'in_progress' || game.current_status === 'waiting_for_opponent'}
                    <button class="primary small" type="button" onclick={() => goto(`/games/${game.id}`)}>Play</button>
                  {/if}
                </div>
              </article>
            {/each}
          </div>
        {/if}
      </section>
    </section>
  </main>
{/if}
