<script>
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { getApi } from '$lib/api.js';
  import { token, userPermission } from '$lib/stores/userStore.js';
  import { goto } from '$app/navigation';
  import { Trash2 } from 'lucide-svelte';

  let users = $state([]);

  onMount(async () => {
    if ($userPermission < 3) { goto('/'); return; }
    const res = await fetch(`${getApi()}/getusers`, { headers: { Authorization: $token } });
    if (res.ok) users = await res.json();
  });

  async function changeRole(userId, newRole) {
    await fetch(`${getApi()}/updateuser`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: $token },
      body: JSON.stringify({ id: userId, role: newRole }),
    });
    const i = users.findIndex(u => u._id === userId);
    if (i >= 0) users[i].role = newRole;
    users = [...users];
  }

  async function deleteUser(userId) {
    if (!confirm('Delete this user?')) return;
    await fetch(`${getApi()}/deleteuser?id=${userId}`, {
      method: 'DELETE',
      headers: { Authorization: $token },
    });
    users = users.filter(u => u._id !== userId);
  }
</script>

<svelte:head><title>Manage Users — Blindtest</title></svelte:head>

<div class="manage-page">
  <div class="page-header">
    <h1>Manage Users</h1>
  </div>
  <table>
    <thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Registered</th><th>Actions</th></tr></thead>
    <tbody>
      {#each users as u (u._id)}
        <tr class:deleted={u.deleted} transition:fade={{ duration: 200 }}>
          <td class="user-name">{u.name}</td>
          <td>{u.email}</td>
          <td>
            <select value={u.role} onchange={(e) => changeRole(u._id, e.target.value)}>
              <option value="user">User</option>
              <option value="contributor">Contributor</option>
              <option value="administrator">Administrator</option>
            </select>
          </td>
          <td class="mono">{new Date(u.registerDate).toLocaleDateString()}</td>
          <td>
            {#if !u.deleted}
              <button class="btn-danger sm" onclick={() => deleteUser(u._id)}><Trash2 size={13} stroke-width={1.8} /></button>
            {:else}
              <span class="deleted-label">Deleted</span>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .manage-page { padding: 28px 24px; overflow: auto; width: 100%; }
  .page-header {
    margin-bottom: 24px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--border);
  }
  h1 {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }
  table { width: 100%; border-collapse: collapse; background: var(--surface); border-radius: var(--radius-lg); overflow: hidden; border: 1px solid var(--border); box-shadow: var(--shadow-card); }
  th {
    text-align: left;
    padding: 12px 16px;
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--text-dim);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
  }
  td {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    font-size: 0.875rem;
    color: var(--text-secondary);
  }
  td.mono {
    font-family: var(--mono);
    font-size: 0.8rem;
    color: var(--text-dim);
  }
  .user-name {
    font-weight: 600;
    color: var(--text-primary);
  }
  tr:last-child td { border-bottom: none; }
  tr:hover td { background: var(--surface-2); }
  tr.deleted { opacity: 0.35; }
  .deleted-label {
    font-size: 0.75rem;
    color: var(--red);
    font-weight: 500;
  }
  :global(.btn-danger.sm) {
    font-size: 0.75rem;
    padding: 5px 10px;
  }
</style>
