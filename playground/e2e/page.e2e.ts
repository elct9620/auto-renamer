import { type Page, expect, test } from '@playwright/test'

function node(page: Page, name: string) {
  return page.locator('.react-flow__node').filter({ has: page.getByText(name, { exact: true }) })
}

// The configuration tab is named after whatever is being edited, so it is found by its place.
async function configText(page: Page): Promise<string> {
  await page.getByRole('tab').nth(1).click()
  return (await page.getByRole('tabpanel').textContent()) ?? ''
}

async function dragBy(page: Page, name: string, dy: number) {
  const box = (await node(page, name).boundingBox())!
  await page.mouse.move(box.x + 20, box.y + 10)
  await page.mouse.down()
  await page.mouse.move(box.x + 20, box.y + 10 + dy, { steps: 8 })
  await page.mouse.up()
}

async function addFolderConfiguration(page: Page, folder: string) {
  await page.getByRole('combobox', { name: 'Path of a new file or folder' }).first().fill(`${folder}/auto-renamer.toml`)
  await page.getByRole('button', { name: 'Add file' }).first().click()
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
  // A drag may pan the view, so the watch is placed against a pipeline rather than the page.
  const top = async (name: string) => Math.round((await node(page, name).boundingBox())!.y)
  const gap = (await top('video')) - (await top('series'))
  await dragBy(page, 'series', 150)

  await page.getByRole('button', { name: 'Reset layout' }).click()

  await expect.poll(async () => (await top('video')) - (await top('series'))).toBe(gap)
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
test('a folder configuration chosen in the tree is edited with the trees still shown', async ({ page }) => {
  await addFolderConfiguration(page, 'Alpha')

  await page.getByRole('button', { name: 'auto-renamer.toml', exact: true }).click()

  await expect(page.getByText('Folder configuration of /downloads/Alpha')).toBeVisible()
  await expect(page.getByRole('heading', { name: /^Source/ })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^Target/ })).toBeVisible()
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

// @behavior PGE-048
test('a folder configuration edited in the form applies in the next simulation', async ({ page }) => {
  await addFolderConfiguration(page, 'Alpha')
  await page.getByRole('button', { name: 'auto-renamer.toml', exact: true }).click()
  await page.getByRole('button', { name: 'Add a value' }).click()
  const name = page.getByRole('textbox', { name: 'Name' })
  await name.fill('show')
  await name.blur()
  const value = page.getByRole('textbox', { name: 'Value of show' })
  await value.fill('Beta')
  await value.blur()

  await page.getByRole('button', { name: 'Trigger' }).click()

  await expect(page.getByText('Beta s01e01.mkv', { exact: true })).toBeVisible()
})

// @behavior PGE-049
test('going back to the global configuration edits it again', async ({ page }) => {
  await addFolderConfiguration(page, 'Alpha')
  await page.getByRole('button', { name: 'auto-renamer.toml', exact: true }).click()
  await expect(node(page, 'series')).toHaveCount(0)

  await page.getByRole('button', { name: 'Back to the global configuration' }).click()

  await expect(node(page, 'series')).toBeVisible()
})

// @behavior PGE-050
test('an imported folder configuration lands at the source root while the global one is edited', async ({ page }) => {
  await page.locator('input[type=file]').setInputFiles({
    name: 'auto-renamer.toml',
    mimeType: 'application/toml',
    buffer: Buffer.from('[vars]\nshow = "Beta"\n'),
  })

  await expect(page.getByText('Folder configuration of /downloads', { exact: true })).toBeVisible()
  await expect(page.getByRole('button', { name: 'auto-renamer.toml', exact: true })).toBeVisible()
})

// @behavior PGE-051
test('a folder configuration added in a folder is opened', async ({ page }) => {
  await page.getByText('Alpha', { exact: true }).first().hover()

  await page.getByRole('button', { name: 'Add a folder configuration in Alpha' }).click()

  await expect(page.getByRole('button', { name: 'auto-renamer.toml', exact: true })).toBeVisible()
  await expect(page.getByText('Folder configuration of /downloads/Alpha', { exact: true })).toBeVisible()
})

// @behavior PGE-052
test('a stage is shown by its name beside the CLI\'s name', async ({ page }) => {
  await page.getByRole('combobox', { name: 'Language' }).click()
  await page.getByRole('option', { name: '繁體中文' }).click()

  const filter = page.locator('aside').first().locator('span', { has: page.locator('code', { hasText: /^filter$/ }) })

  await expect(filter).toHaveText('篩選filter')
})

// @behavior PGE-053
test('an imported folder configuration replaces the one being edited', async ({ page }) => {
  await addFolderConfiguration(page, 'Alpha')
  await page.getByRole('button', { name: 'auto-renamer.toml', exact: true }).click()

  await page.locator('input[type=file]').setInputFiles({
    name: 'auto-renamer.toml',
    mimeType: 'application/toml',
    buffer: Buffer.from('[vars]\nshow = "Beta"\n'),
  })

  await expect(page.getByRole('button', { name: 'auto-renamer.toml', exact: true })).toHaveCount(1)
  expect(await configText(page)).toContain('show = "Beta"')
})

// @behavior PGE-054
test('a download is the configuration being edited', async ({ page }) => {
  await addFolderConfiguration(page, 'Alpha')
  await page.getByRole('button', { name: 'auto-renamer.toml', exact: true }).click()

  const download = page.waitForEvent('download')
  await page.getByRole('button', { name: 'Download' }).click()

  expect((await download).suggestedFilename()).toBe('auto-renamer.toml')
})
