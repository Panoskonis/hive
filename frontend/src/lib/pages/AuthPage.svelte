<script lang="ts">
  import { goto } from '$app/navigation'
  import { onMount } from 'svelte'
  import Topbar from '$lib/components/Topbar.svelte'
  import { readableError } from '$lib/hive-ui'
  import { api } from '$lib/session.svelte'

  let { mode }: { mode: 'login' | 'register' } = $props()

  let username = $state('')
  let email = $state('')
  let password = $state('')
  let authError = $state('')
  let authBusy = $state(false)

  onMount(() => {
    if (api.isAuthenticated) {
      void goto('/dashboard')
    }
  })

  async function submitAuth() {
    authBusy = true
    authError = ''

    try {
      if (mode === 'login') {
        await api.login(email, password)
      } else {
        await api.register(username, email, password)
      }
      password = ''
      await goto('/dashboard')
    } catch (error) {
      authError = readableError(error)
    } finally {
      authBusy = false
    }
  }
</script>

<main class="public-shell">
  <Topbar publicNav />

  <section class="auth-view">
    <form class="auth-panel" onsubmit={(event) => { event.preventDefault(); submitAuth() }}>
      <div>
        <p>{mode === 'login' ? 'Welcome back' : 'New player'}</p>
        <h1>{mode === 'login' ? 'Login' : 'Register'}</h1>
      </div>

      {#if mode === 'register'}
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
        <input
          bind:value={password}
          type="password"
          autocomplete={mode === 'login' ? 'current-password' : 'new-password'}
          required
        />
      </label>

      {#if authError}
        <p class="error">{authError}</p>
      {/if}

      <button class="primary" type="submit" disabled={authBusy}>
        {authBusy ? 'Working...' : mode === 'login' ? 'Login' : 'Create account'}
      </button>

      <button class="text-button" type="button" onclick={() => goto(mode === 'login' ? '/register' : '/login')}>
        {mode === 'login' ? 'Need an account?' : 'Already registered?'}
      </button>
    </form>
  </section>
</main>
