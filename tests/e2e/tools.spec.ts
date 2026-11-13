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

const wasmModules = [
  'simplex',
  'linalg',
  'complexnum',
  'graphnet',
  'series',
  'logic',
  'floating',
  'boolean'
];

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

test('detects the general term of sin x', async ({ page }) => {
  await page.goto('/series');
  await page.getByRole('button', { name: 'Expand', exact: true }).click();
  await expect(page.locator('.status')).toHaveText('Maclaurin series');
  await expect(page.locator('.general-term')).toBeVisible();
  await expect(page.locator('.general-term')).toContainText('General term');
});

test('accepts ln(1-3x) and offers function keys', async ({ page }) => {
  await page.goto('/series');
  await expect(page.getByRole('button', { name: 'sine', exact: true })).toBeVisible();
  const field = page.locator('math-field').first();
  await field.waitFor();
  await field.evaluate((element) => {
    const input = element as HTMLElement & { value: string };
    input.value = '\\ln\\left(1-3x\\right)';
    input.dispatchEvent(new Event('input', { bubbles: true }));
  });
  await page.getByRole('button', { name: 'Expand', exact: true }).click();
  await expect(page.locator('.error')).toHaveCount(0);
  await expect(page.locator('.status')).toHaveText('Maclaurin series');
});

test('typing ln(1-3x) into the field works', async ({ page }) => {
  await page.goto('/series');
  const field = page.locator('math-field').first();
  await field.waitFor();
  await field.evaluate((element) => {
    const input = element as HTMLElement & { value: string };
    input.value = '';
    input.dispatchEvent(new Event('input', { bubbles: true }));
  });
  await field.focus();
  await page.keyboard.type('ln(1-3x)');
  const latex = await field.evaluate((element) => (element as HTMLElement & { getValue: (format: string) => string }).getValue('latex'));
  expect(latex, 'MathLive should contain the typed expression').not.toBe('');
  await page.getByRole('button', { name: 'Expand', exact: true }).click();
  const errorText = await page
    .locator('.error')
    .textContent({ timeout: 1000 })
    .catch(() => null);
  expect(errorText, `field latex: ${latex}`).toBeNull();
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
