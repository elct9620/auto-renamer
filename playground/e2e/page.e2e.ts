import { type Page, expect, test } from '@playwright/test'

function node(page: Page, name: string) {
  return page.locator('.react-flow__node', { hasText: new RegExp(`^${name}`) })
}

async function configText(page: Page): Promise<string> {
  await page.getByRole('tab', { name: 'config.toml' }).click()
  return (await page.getByRole('tabpanel').textContent()) ?? ''
}

async function dragBy(page: Page, name: string, dy: number) {
  const box = (await node(page, name).boundingBox())!
  await page.mouse.move(box.x + 20, box.y + 10)
  await page.mouse.down()
  await page.mouse.move(box.x + 20, box.y + 10 + dy, { steps: 8 })
  await page.mouse.up()
}

test.beforeEach(async ({ page }) => {
  await page.goto('/')
  await expect(node(page, 'series')).toBeVisible()
})

// @behavior PGE-032
test('a node moved on the canvas leaves the configuration as it was', async ({ page }) => {
  const before = await configText(page)

  await dragBy(page, 'series', 150)

  expect(await configText(page)).toBe(before)
})

// @behavior PGE-033
test('a reset layout draws every node where its order places it', async ({ page }) => {
  const first = (await node(page, 'series').boundingBox())!
  await dragBy(page, 'series', 150)

  await page.getByRole('button', { name: 'Reset layout' }).click()

  await expect.poll(async () => Math.round((await node(page, 'series').boundingBox())!.y)).toBe(Math.round(first.y))
})

// @behavior PGE-034
test('restoring the example brings back its configuration and tree', async ({ page }) => {
  const before = await configText(page)
  await node(page, 'move').click()
  await page.getByRole('button', { name: 'Remove', exact: true }).click()
  const path = page.getByRole('combobox', { name: 'Path of a new file or folder' }).first()
  await path.fill('Beta/01.mkv')
  await page.getByRole('button', { name: 'Add file' }).first().click()

  await page.getByRole('button', { name: 'Restore example' }).click()

  expect(await configText(page)).toBe(before)
  await expect(page.getByText('01.mkv', { exact: true })).toHaveCount(0)
})

// @behavior PGE-035
test('adding in a folder starts the new path from that folder', async ({ page }) => {
  await page.getByText('Season 1', { exact: true }).hover()

  await page.getByRole('button', { name: 'Add in Season 1' }).click()

  await expect(page.getByRole('combobox', { name: 'Path of a new file or folder' }).first()).toHaveValue('Alpha/Season 1/')
})

// @behavior PGE-036
test('a parameter limited to some values offers them in a list', async ({ page }) => {
  await node(page, 'move').click()

  await page.getByRole('combobox', { name: 'On a clash' }).click()

  await expect(page.getByRole('option')).toHaveText(['—', 'reject', 'suffix'])
})

// @behavior PGE-037
test('a required parameter left empty is pointed out', async ({ page }) => {
  await node(page, 'video').click()
  await page.getByRole('combobox', { name: 'Add a stage' }).click()
  await page.getByRole('option', { name: 'next' }).click()
  await node(page, 'next').click()
  const into = page.getByRole('textbox', { name: /^Target field into/ })
  await into.fill('')

  await into.blur()

  await expect(page.getByText('Required')).toBeVisible()
})

// @behavior PGE-038
test('a folder configuration shows no virtual tree', async ({ page }) => {
  await page.getByRole('combobox', { name: 'Configuration kind' }).click()

  await page.getByRole('option', { name: 'Folder configuration' }).click()

  await expect(page.getByRole('heading', { name: /^Source/ })).toHaveCount(0)
  await expect(page.getByRole('heading', { name: /^Target/ })).toHaveCount(0)
})

// @behavior PGE-039
test('the language chosen in the header is the one the page speaks', async ({ page }) => {
  await page.getByRole('combobox', { name: 'Language' }).click()

  await page.getByRole('option', { name: '繁體中文' }).click()

  await expect(page.getByRole('button', { name: '恢復範例' })).toBeVisible()
  await expect(page.locator('html')).toHaveAttribute('lang', 'zh-TW')
})

// @behavior PGE-040
test('settings the simulation ignores are marked as not simulated', async ({ page }) => {
  await node(page, 'series').click()

  const marked = page.getByText('not simulated')

  await expect(marked).toHaveCount(2)
})

// @behavior PGE-041
test('a watch lists pipelines picked from those defined', async ({ page }) => {
  await page.getByRole('button', { name: 'Add pipeline' }).click()
  await node(page, 'series').click()

  await page.getByRole('combobox', { name: 'Add a pipeline' }).click()
  await page.getByRole('option', { name: 'new_pipeline' }).click()

  expect(await configText(page)).toContain('pipelines = ["video", "new_pipeline"]')
})

// @behavior PGE-042
test('a unit is chosen among its three forms', async ({ page }) => {
  await node(page, 'series').click()

  await page.getByRole('combobox', { name: 'Unit' }).click()

  await expect(page.getByRole('option')).toHaveText(['—', /^directory/, /^source/, /^root/])
})

// @behavior PGE-043
test('a yes-or-no setting is a switch', async ({ page }) => {
  await node(page, 'series').click()

  const dryRun = page.getByRole('switch', { name: 'Dry run dry_run' })

  await expect(dryRun).toBeVisible()
})

// @behavior PGE-044
test('a key is labelled in the page\'s language beside the CLI\'s key', async ({ page }) => {
  await page.getByRole('combobox', { name: 'Language' }).click()
  await page.getByRole('option', { name: '繁體中文' }).click()

  await node(page, 'series').click()

  await expect(page.getByRole('switch', { name: '試運行 dry_run' })).toBeVisible()
})

// @behavior PGE-045
test('the canvas sits between the palette and the inspector, above the trees', async ({ page }) => {
  const box = async (locator: ReturnType<Page['locator']>) => (await locator.boundingBox())!

  const palette = await box(page.getByRole('button', { name: 'Add pipeline' }))
  const canvas = await box(page.locator('.react-flow'))
  const inspector = await box(page.getByRole('heading', { name: 'Defaults' }))
  const source = await box(page.getByRole('heading', { name: /^Source/ }))
  const target = await box(page.getByRole('heading', { name: /^Target/ }))

  expect(palette.x).toBeLessThan(canvas.x)
  expect(canvas.x + canvas.width).toBeLessThanOrEqual(inspector.x)
  expect(source.y).toBeGreaterThan(canvas.y + canvas.height - 1)
  expect(source.x).toBeLessThan(target.x)
})
