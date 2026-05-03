<script>
  import { onMount } from 'svelte';
  import { getApi } from '$lib/api.js';
  import { token, userPermission } from '$lib/stores/userStore.js';
  import { goto } from '$app/navigation';

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
        <tr class:deleted={u.deleted}>
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
              <button class="btn-danger sm" onclick={() => deleteUser(u._id)}>🗑</button>
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
    margin-bottom: 20px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--border);
  }
  h1 {
    font-family: var(--mono);
    font-size: 1rem;
    font-weight: 500;
    letter-spacing: -0.02em;
  }
  table { width: 100%; border-collapse: collapse; }
  th {
    text-align: left;
    padding: 10px 12px;
    font-family: var(--mono);
    font-size: 0.62rem;
    color: var(--text-dim);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    border-bottom: 1px solid var(--border);
  }
  td {
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  td.mono {
    font-family: var(--mono);
    font-size: 0.75rem;
    color: var(--text-dim);
  }
  .user-name {
    font-weight: 500;
    color: var(--text-primary);
  }
  tr:hover { background: var(--surface); }
  tr.deleted { opacity: 0.35; }
  .deleted-label {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--red);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  :global(.btn-danger.sm) {
    font-size: 0.6rem;
    padding: 3px 8px;
  }
</style>
