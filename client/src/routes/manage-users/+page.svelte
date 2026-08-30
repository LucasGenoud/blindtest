<script>
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { getApi } from '$lib/api.js';
  import { token, userPermission } from '$lib/stores/userStore.js';

  let users = $state([]);

  onMount(async () => {
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
              <button class="btn-danger sm" onclick={() => deleteUser(u._id)}>Delete</button>
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
  .manage-page { padding: 32px; overflow: auto; width: 100%; }

  .page-header {
    margin-bottom: 24px;
    padding-bottom: 12px;
    border-bottom: 2px solid var(--divider);
  }

  h1 {
    font-size: 32px;
    font-weight: 800;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  /* A ruled list, not a filled panel: no fill, no border, no shadow. */
  table {
    width: 100%;
    border-collapse: collapse;
    background: transparent;
    border-radius: 0;
    border: 0;
  }

  th {
    text-align: left;
    padding: 8px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    background: transparent;
    border-bottom: 2px solid var(--divider);
    color: var(--text-primary);
  }

  td {
    padding: 8px;
    border-bottom: 1px solid var(--divider);
    font-size: 13px;
    color: var(--text-primary);
  }

  td.mono {
    font-size: 13px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }

  .user-name {
    font-weight: 600;
    color: var(--text-primary);
  }

  tr:hover td { background: rgba(245, 241, 232, 0.06); }
  tr.deleted { opacity: 0.35; }

  .deleted-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--signal-wrong);
  }

  :global(.btn-danger.sm) {
    font-size: 13px;
    padding: 4px 8px;
    min-height: 28px;
  }
</style>
