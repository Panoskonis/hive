import { goto } from '$app/navigation'
import { ApiClient } from './api.svelte'

export const api = new ApiClient()

export async function signOut() {
  await api.logout()
  await goto('/')
}
