import { openFileList } from '../../fileSwitcherHelpers'
import { When } from './fixtures'

/** Selects a file of the cloud-backed list by its path (extension left off),
 * expanding each folder on the way, then leaves the file list open. */
When(
  'I open the cloud file {string} from the file list',
  async ({ page }, path: string) => {
    await openFileList(page)
    const segments = path.split('/')
    // Filter to this folder before selecting a leaf: other scenarios can
    // seed the same leaf name in other folders of the shared account. Search
    // expands matching folders automatically.
    const search = page.getByRole('searchbox', { name: 'Search files' })
    await search.fill(segments[0] ?? path)
    const fileName = segments.at(-1) ?? path
    await page
      .locator('.file-tab-name', { hasText: new RegExp(`^${fileName}$`) })
      .click()
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await openFileList(page)
    await search.clear()
  },
)
