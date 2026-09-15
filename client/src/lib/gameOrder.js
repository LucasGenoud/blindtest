export function allocateCategories(weights, count) {
  const active = Object.entries(weights).filter(([, weight]) => weight > 0);
  const total = active.reduce((sum, [, weight]) => sum + weight, 0);
  if (!total || count <= 0) return [];

  const shares = active.map(([category, weight]) => {
    const exact = count * weight / total;
    return { category, count: Math.floor(exact), remainder: exact % 1 };
  });
  let left = count - shares.reduce((sum, share) => sum + share.count, 0);
  shares.sort((a, b) => b.remainder - a.remainder);
  for (let i = 0; i < left; i++) shares[i].count++;
  return shares.flatMap(({ category, count }) => Array(count).fill(category));
}
