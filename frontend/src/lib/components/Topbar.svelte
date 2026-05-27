<script lang="ts">
  import { goto } from '$app/navigation'

  let {
    userName = '',
    publicNav = false,
    onBrand = () => goto('/'),
    onSignOut,
    showGames = false,
    onGames,
  }: {
    userName?: string
    publicNav?: boolean
    onBrand?: () => void | Promise<void>
    onSignOut?: () => void | Promise<void>
    showGames?: boolean
    onGames?: () => void | Promise<void>
  } = $props()
</script>

<header class={`topbar ${publicNav ? 'public' : ''}`}>
  <button class="brand" type="button" onclick={onBrand}>
    <span class="brand-mark">H</span>
    <span>Hive</span>
  </button>

  {#if publicNav}
    <nav>
      <button class="ghost" type="button" onclick={() => goto('/login')}>Login</button>
      <button class="primary small" type="button" onclick={() => goto('/register')}>Register</button>
    </nav>
  {:else}
    <div class="session">
      <span>{userName}</span>
      {#if showGames && onGames}
        <button class="ghost" type="button" onclick={onGames}>Games</button>
      {/if}
      {#if onSignOut}
        <button class="ghost" type="button" onclick={onSignOut}>Sign out</button>
      {/if}
    </div>
  {/if}
</header>
