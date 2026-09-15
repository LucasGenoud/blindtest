import assert from 'node:assert/strict';
import test from 'node:test';
import { allocateCategories } from './gameOrder.js';

test('allocates leftovers by weight instead of uniformly', () => {
  const order = allocateCategories({ common: 100, rare: 1 }, 10);
  assert.deepEqual(order, Array(10).fill('common'));
});

test('returns exactly the requested number of categories', () => {
  assert.equal(allocateCategories({ a: 1, b: 1, c: 1 }, 20).length, 20);
  assert.deepEqual(allocateCategories({ a: 0 }, 20), []);
});
