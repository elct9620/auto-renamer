import { type Page, expect, test } from '@playwright/test'

function node(page: Page, name: string) {
  return page.locator('.react-flow__node').filter({ has: page.getByText(name, { exact: true }) })
}

// The opening example routes the built-in `series-video`, which is copied in before its stages can change.
async function ownSeriesVideo(page: Page) {
  await title(page, 'series-video').click()
  await page.getByRole('button', { name: 'Copy as own pipeline' }).click()
}

// The route the opening example's watch claims with first.
const FIRST_ROUTE = '1 · series-video'

// A watch or a pipeline holds other nodes, so it is chosen by its title rather than its middle.
function title(page: Page, name: string) {
  return node(page, name).getByText(name, { exact: true })
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

// The opening example renames in place, so the target shows the same tree; counts are taken in the source.
function source(page: Page) {
  return page.getByRole('group', { name: 'Source' })
}

// The opening example names every show by its folder, so a folder configuration for Alpha is started and opened.
async function openAlphaConfiguration(page: Page) {
  await source(page).getByRole('button', { name: 'Add a folder configuration in Alpha' }).click()
}

async function chooseExample(page: Page, name: string) {
  await page.getByRole('combobox', { name: 'Examples' }).click()
  await page.getByRole('option', { name }).click()
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
  const gap = (await top('series-video')) - (await top('series'))
  await dragBy(page, 'series', 150)

  await page.getByRole('button', { name: 'Reset layout' }).click()

  await expect.poll(async () => (await top('series-video')) - (await top('series'))).toBe(gap)
})

// @behavior PGE-034
test('choosing the example again brings back its configuration and tree', async ({ page }) => {
  const before = await configText(page)
  await ownSeriesVideo(page)
  await title(page, 'format').click()
  await page.getByRole('button', { name: 'Remove', exact: true }).click()
  const path = page.getByRole('combobox', { name: 'Path of a new file or folder' }).first()
  await path.fill('Omega/01.mkv')
  await page.getByRole('button', { name: 'Add file' }).first().click()

  await chooseExample(page, 'Single episodes')

  expect(await configText(page)).toBe(before)
  await expect(page.getByText('01.mkv', { exact: true })).toHaveCount(0)
})

// @behavior PGE-035
test('adding in a folder starts the new path from that folder', async ({ page }) => {
  await page.getByText('Season 03', { exact: true }).first().hover()

  await page.getByRole('button', { name: 'Add in Season 03' }).first().click()

  await expect(page.getByRole('combobox', { name: 'Path of a new file or folder' }).first()).toHaveValue('Zeta-Show/Season 03/')
})

// @behavior PGE-036
test('a parameter limited to some values offers them in a list', async ({ page }) => {
  await ownSeriesVideo(page)
  await page.getByRole('combobox', { name: 'Add a stage' }).click()
  await page.getByRole('option', { name: 'case' }).click()
  await title(page, 'case').click()

  await page.getByRole('combobox', { name: /^Case/ }).click()

  await expect(page.getByRole('option')).toHaveText(['—', 'lower', 'upper', 'title'])
})

// @behavior PGE-037
test('a required parameter left empty is pointed out', async ({ page }) => {
  await ownSeriesVideo(page)
  await page.getByRole('combobox', { name: 'Add a stage' }).click()
  await page.getByRole('option', { name: 'rank' }).click()
  await title(page, 'rank').click()
  const into = page.getByRole('textbox', { name: /^Target field into/ })
  await into.fill('')

  await into.blur()

  await expect(page.getByText('Required')).toBeVisible()
})

// @behavior PGE-038
test('a folder configuration chosen in the tree is edited with the trees still shown', async ({ page }) => {
  await openAlphaConfiguration(page)

  await expect(page.getByText('Folder configuration of /downloads/Alpha')).toBeVisible()
  await expect(page.getByRole('heading', { name: /^Source/ })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^Target/ })).toBeVisible()
})

// @behavior PGE-039
test('the language chosen in the header is the one the page speaks', async ({ page }) => {
  await page.getByRole('combobox', { name: 'Language' }).click()

  await page.getByRole('option', { name: '繁體中文' }).click()

  await expect(page.getByRole('combobox', { name: '範例' })).toBeVisible()
  await expect(page.locator('html')).toHaveAttribute('lang', 'zh-TW')
})

// @behavior PGE-040
test('a watch offers the settings for when a unit is processed', async ({ page }) => {
  await title(page, 'series').click()

  await expect(page.getByRole('textbox', { name: 'Quiet for quiet' })).toBeVisible()
  await expect(page.getByRole('textbox', { name: 'Wait at most max_wait' })).toBeVisible()
  await expect(page.getByRole('spinbutton', { name: 'Most files max_files' })).toBeVisible()
})

// @behavior PGE-041
test('a watch routes pipelines picked from those defined', async ({ page }) => {
  await page.getByRole('button', { name: 'Add pipeline' }).click()
  await title(page, 'series').click()

  await page.getByRole('combobox', { name: 'Add a pipeline' }).click()
  await page.getByRole('option', { name: 'new_pipeline' }).click()

  const text = await configText(page)
  expect(text).toContain('[[watch.series.routes]]')
  expect(text).toContain('pipeline = "new_pipeline"')
})

// @behavior PGE-042
test('a unit is chosen among its three forms', async ({ page }) => {
  await title(page, 'series').click()

  await page.getByRole('combobox', { name: 'Unit' }).click()

  await expect(page.getByRole('option')).toHaveText(['—', /^directory/, /^source/, /^root/])
})

// @behavior PGE-043
test('a yes-or-no setting is a switch', async ({ page }) => {
  await title(page, 'series').click()

  const dryRun = page.getByRole('switch', { name: 'Dry run dry_run' })

  await expect(dryRun).toBeVisible()
})

// @behavior PGE-044
test('a key is labelled in the page\'s language beside the CLI\'s key', async ({ page }) => {
  await page.getByRole('combobox', { name: 'Language' }).click()
  await page.getByRole('option', { name: '繁體中文' }).click()

  await title(page, 'series').click()

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
  await openAlphaConfiguration(page)
  await page.getByRole('button', { name: 'Add a value' }).click()
  const name = page.getByRole('textbox', { name: 'Name', exact: true })
  await name.fill('show')
  await name.blur()
  const value = page.getByRole('textbox', { name: 'Value of show' })
  await value.fill('Omega')
  await value.blur()

  await page.getByRole('button', { name: 'Trigger' }).click()

  await expect(page.getByText('Omega s01e12.mkv', { exact: true })).toBeVisible()
})

// @behavior PGE-049
test('going back to the global configuration edits it again', async ({ page }) => {
  await openAlphaConfiguration(page)
  await expect(node(page, 'series')).toHaveCount(0)

  await page.getByRole('combobox', { name: 'Configuration being edited' }).click()
  await page.getByRole('option', { name: 'Global configuration' }).click()

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
  await expect(source(page).getByRole('button', { name: 'auto-renamer.toml', exact: true })).toHaveCount(1)
})

// @behavior PGE-051
test('a folder configuration added in a folder is opened', async ({ page }) => {
  await page.getByText('Alpha', { exact: true }).first().hover()

  await page.getByRole('button', { name: 'Add a folder configuration in Alpha' }).first().click()

  await expect(source(page).getByRole('button', { name: 'auto-renamer.toml', exact: true })).toHaveCount(1)
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
  await openAlphaConfiguration(page)

  await page.locator('input[type=file]').setInputFiles({
    name: 'auto-renamer.toml',
    mimeType: 'application/toml',
    buffer: Buffer.from('[vars]\nshow = "Beta"\n'),
  })

  await expect(source(page).getByRole('button', { name: 'auto-renamer.toml', exact: true })).toHaveCount(1)
  expect(await configText(page)).toContain('show = "Beta"')
})

// @behavior PGE-054
test('a download is the configuration being edited', async ({ page }) => {
  await openAlphaConfiguration(page)

  const download = page.waitForEvent('download')
  await page.getByRole('button', { name: 'Download' }).click()

  expect((await download).suggestedFilename()).toBe('auto-renamer.toml')
})

// @behavior PGE-055
test('the page opens on the single-episode example', async ({ page }) => {
  const tree = source(page)

  await expect(tree.getByText('Zeta-Show', { exact: true })).toBeVisible()
  await expect(tree.getByText('[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv', { exact: true })).toBeVisible()
})

// @behavior PGE-058
test('a joint\'s remove button drops the pipeline from the watch', async ({ page }) => {
  await page.getByRole('button', { name: 'Remove series-video from series' }).click()

  await expect(page.getByRole('button', { name: 'Remove series-video from series' })).toHaveCount(0)
  expect(await configText(page)).toContain('routes = []')
})

// @behavior PGE-060
test('adding a folder configuration is offered without hovering', async ({ page }) => {
  const tree = source(page)

  const button = tree.getByRole('button', { name: 'Add a folder configuration in Alpha' })

  // Playwright counts a transparent element as visible, so the opacity of the row part holding it is checked.
  await expect(button).toBeVisible()
  await expect(button.locator('..')).toHaveCSS('opacity', '1')
  await expect(tree.getByText('To make a folder an exception', { exact: false })).toBeVisible()
})

// @behavior PGE-061
test('the header lists every folder configuration to edit', async ({ page }) => {
  await openAlphaConfiguration(page)

  await page.getByRole('combobox', { name: 'Configuration being edited' }).click()

  await expect(page.getByRole('option', { name: 'Global configuration' })).toBeVisible()
  await expect(page.getByRole('option', { name: /^Folder configuration of / })).toHaveCount(1)
  await expect(page.getByRole('option', { name: 'Folder configuration of /downloads/Alpha' })).toBeVisible()
})

// @behavior PGE-062
test('a folder configuration added from the header is started and opened', async ({ page }) => {
  await page.getByRole('combobox', { name: 'Configuration being edited' }).click()

  await page.getByRole('option', { name: 'Add a folder configuration in /downloads/Alpha', exact: true }).click()

  await expect(source(page).getByRole('button', { name: 'auto-renamer.toml', exact: true })).toHaveCount(1)
  await expect(page.getByRole('combobox', { name: 'Configuration being edited' })).toHaveText('Folder configuration of /downloads/Alpha')
})

// @behavior PGE-064
test('overriding a pipeline in a folder opens that folder\'s configuration', async ({ page }) => {
  await title(page, 'series-video').click()

  await page.getByRole('combobox', { name: /Override in a folder/ }).click()
  await page.getByRole('option', { name: '/downloads/Alpha', exact: true }).click()

  await expect(page.getByRole('combobox', { name: 'Configuration being edited' })).toHaveText('Folder configuration of /downloads/Alpha')
  await expect(node(page, 'series-video')).toBeVisible()
  await expect(node(page, 'series')).toHaveCount(0)
})

// @behavior PGE-066
test('choosing a result shows how its file was planned', async ({ page }) => {
  await page.getByRole('button', { name: 'Trigger' }).click()

  await page.getByRole('button', { name: 'Alpha/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv' }).click()

  const steps = page.getByRole('list', { name: /^Steps of / }).getByRole('listitem')
  await expect(steps.first()).toHaveText('Claimed by series-video')
  await expect(steps.filter({ hasText: 'episode: 12' })).toHaveCount(1)
  await expect(steps.last()).toContainText('move')
})

// @behavior PGE-068
test('a selected stage shows the files after it', async ({ page }) => {
  await page.getByRole('button', { name: 'Trigger' }).click()

  await title(page, 'format').click()

  const after = page.getByRole('region', { name: 'Files after this stage' })
  await expect(after.getByRole('listitem')).toHaveCount(6)
  await expect(after.getByText('name: Alpha s01e12', { exact: true })).toBeVisible()
})

// @behavior PGE-069
test('editing the configuration clears the simulation', async ({ page }) => {
  await ownSeriesVideo(page)
  await page.getByRole('button', { name: 'Trigger' }).click()
  await expect(page.getByText('Trigger the watch to see where each file goes.')).toHaveCount(0)

  await title(page, 'format').click()
  await page.getByRole('button', { name: 'Remove', exact: true }).click()

  await expect(page.getByText('Trigger the watch to see where each file goes.')).toBeVisible()
})

async function declareTarget(page: Page, name: string) {
  const field = page.getByRole('textbox', { name: 'New target target' })
  await field.fill(name)
  await field.blur()
}

// @behavior PGE-071
test('a target added in the defaults form is declared', async ({ page }) => {
  await declareTarget(page, 'conflict')

  expect(await configText(page)).toContain('[target.conflict]')
})

// @behavior PGE-072
test('a route moves into a target picked from those declared', async ({ page }) => {
  await declareTarget(page, 'conflict')
  await title(page, FIRST_ROUTE).click()

  await page.getByRole('combobox', { name: 'Move to', exact: true }).click()
  await page.getByRole('option', { name: 'conflict' }).click()

  expect(await configText(page)).toContain('move = "conflict"')
})

// @behavior PGE-083
test('a refused file is sent through a pipeline picked from those defined', async ({ page }) => {
  await title(page, FIRST_ROUTE).click()

  await page.getByRole('combobox', { name: 'Refused files run' }).click()
  await page.getByRole('option', { name: 'series-subtitle' }).click()

  expect(await configText(page)).toContain('pipeline = "series-subtitle"')
})

// @behavior PGE-084
test('a route that cleans up keeps the folders it lists', async ({ page }) => {
  // The opening route already cleans up, so it is first set not to.
  await title(page, FIRST_ROUTE).click()
  const cleanup = page.getByRole('switch', { name: /^Clean up/ })
  await cleanup.click()
  expect(await configText(page)).not.toContain('cleanup')
  await title(page, FIRST_ROUTE).click()

  await page.getByRole('switch', { name: /^Clean up/ }).click()
  const keep = page.getByRole('textbox', { name: /^Keep/ })
  await keep.fill('Extras')
  await keep.blur()

  expect(await configText(page)).toMatch(/\[watch\.series\.routes\.cleanup\]\s+keep = \["Extras"\]/)
})

// @behavior PGE-089
test('a default route is edited in full in the defaults form', async ({ page }) => {
  await declareTarget(page, 'conflict')

  await page.getByRole('combobox', { name: 'Add a pipeline' }).click()
  await page.getByRole('option', { name: 'series-video' }).click()
  await page.getByRole('combobox', { name: 'Move to', exact: true }).click()
  await page.getByRole('option', { name: 'conflict' }).click()

  expect(await configText(page)).toMatch(/\[\[default\.routes\]\]\s+move = "conflict"\s+pipeline = "series-video"/)
})

// @behavior PGE-092
test('an imported global configuration replaces the one being edited', async ({ page }) => {
  await page.locator('input[type=file]').setInputFiles({
    name: 'config.toml',
    mimeType: 'application/toml',
    buffer: Buffer.from('[pipeline.p]\nstages = []\n\n[watch.movies]\nsource = "/movies"\nroutes = [{ pipeline = "p" }]\n'),
  })

  const text = await configText(page)
  expect(text).toContain('[watch.movies]')
  expect(text).not.toContain('[watch.series]')
})

// @behavior PGE-093
test('a removed watch is written nowhere', async ({ page }) => {
  await title(page, 'series').click()

  await page.getByRole('button', { name: 'Remove watch' }).click()

  expect(await configText(page)).not.toContain('[watch.series]')
})

// @behavior PGE-094
test('a selected target\'s path is edited in its form', async ({ page }) => {
  await declareTarget(page, 'conflict')
  await title(page, 'conflict').click()
  const path = page.getByRole('textbox', { name: /^Path/ })

  await path.fill('/clash')
  await path.blur()

  expect(await configText(page)).toMatch(/\[target\.conflict\]\s+path = "\/clash"/)
})

// @behavior PGE-095
test('a refused file moves into a target picked from those declared', async ({ page }) => {
  await declareTarget(page, 'conflict')
  await title(page, FIRST_ROUTE).click()

  await page.getByRole('combobox', { name: 'Refused files move to' }).click()
  await page.getByRole('option', { name: 'conflict' }).click()

  expect(await configText(page)).toMatch(/\[watch\.series\.routes\.rejected\]\s+move = "conflict"/)
})
