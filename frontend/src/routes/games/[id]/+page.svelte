<script lang="ts">
  import GamePage from '$lib/pages/GamePage.svelte'

  let { data } = $props()
</script>

<svelte:head>
  <title>Game #{data.gameLabel} | Hive</title>
</svelte:head>

<GamePage gameId={data.gameId} />
