<script lang="ts">
  import { goto } from '$app/navigation'
  import Topbar from '$lib/components/Topbar.svelte'
</script>

<main class="public-shell">
  <Topbar publicNav />

  <section class="hero">
    <div class="hero-copy">
      <p>Two-player strategy</p>
      <h1>HIVE Board Game</h1>
      <div class="hero-actions">
        <button class="primary" type="button" onclick={() => goto('/register')}>Create account</button>
        <button class="secondary" type="button" onclick={() => goto('/login')}>Login</button>
      </div>
    </div>
    <div class="board-preview" aria-hidden="true">
      {#each Array(19) as _, index}
        <span class:dark={index % 3 === 0} class:gold={index % 4 === 0}></span>
      {/each}
    </div>
  </section>
</main>
