import { expect, type Page } from '@playwright/test';

export class FilterModalPO {
  constructor(private page: Page) {}

  private get root() {
    // AntD always renders the visible dialog with role="dialog"
    return this.page.getByRole('dialog', { name: 'Filters' }).last();
  }

  async waitReady() {
    await expect(this.root).toBeVisible();
    await this.root.locator('input,textarea,[contenteditable="true"],button')
      .first()
      .waitFor({ state: 'visible' });
  }

  async expandGroup(label: string | RegExp) {
    await this.root.getByTestId(`filter-group-${String(label).toLowerCase().replace(/\s+/g,'_')}-label`)
      .or(this.root.getByText(label))
      .first()
      .click();
  }

  async setSelect(key: string, optionText: string | RegExp) {
    const trigger = this.root.getByTestId(`filter-${key}-select`)
      .or(this.root.getByTestId(`filter-${key}-multi-select`))
      .first();
    await expect(trigger).toBeVisible();
    await trigger.click();

    const dd = this.page.locator('.ant-select-dropdown:visible').last();
    await expect(dd).toBeVisible();
    const rx = optionText instanceof RegExp ? optionText : new RegExp(`^${optionText}$`, 'i');
    await dd.getByRole('option', { name: rx }).first().click();
  }

  async setText(key: string, value: string) {
    await this.root.getByTestId(`filter-${key}-text`).fill(value);
  }

  async setNumber(key: string, value: string) {
    await this.root.getByTestId(`filter-${key}-number`).fill(value);
  }

  async clear(key: string) {
    await this.root.getByTestId(`filter-${key}-clear`).click();
  }

  async apply() {
    await this.root.getByTestId('filter-apply').click();
  }

  async close() {
    await this.root.getByTestId('filter-close').click();
  }
}
