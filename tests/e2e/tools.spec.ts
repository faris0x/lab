import { expect, test } from '@playwright/test';

const routes = [
  '/',
  '/optimisation',
  '/matrix',
  '/complex',
  '/series',
  '/logic',
  '/graphs',
  '/floating-point',
  '/boolean',
  '/404'
];

const wasmModules = ['simplex', 'linalg', 'complexnum', 'graphnet'];

test.describe('routes', () => {
  for (const route of routes) {
    test(`${route} loads without errors`, async ({ page }) => {
      const errors: string[] = [];
      page.on('console', (message) => {
        if (message.type() === 'error') errors.push(message.text());
      });
      page.on('pageerror', (error) => errors.push(error.message));

      const response = await page.goto(route);
      expect(response?.status()).toBe(200);
      await expect(page.locator('h1')).toBeVisible();
      await page.waitForLoadState('networkidle');
      expect(errors).toEqual([]);
    });
  }
});

test('serves every generated wasm module', async ({ request }) => {
  for (const name of wasmModules) {
    const response = await request.get(`/wasm/${name}/${name}_bg.wasm`);
    expect(response.status(), name).toBe(200);
  }
});

test('solves the default simplex problem', async ({ page }) => {
  await page.goto('/optimisation');
  await page.getByRole('button', { name: 'Solve', exact: true }).click();
  await expect(page.locator('.status')).toHaveText('Optimal', {
    timeout: 30_000
  });
});

test('expands a Maclaurin series', async ({ page }) => {
  await page.goto('/series');
  await page.getByRole('button', { name: 'Expand', exact: true }).click();
  await expect(page.locator('.status')).toHaveText('Maclaurin series');
});

test('home page does not overflow on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 360, height: 800 });
  await page.goto('/');
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth
  );
  expect(overflow).toBeLessThanOrEqual(1);
});
