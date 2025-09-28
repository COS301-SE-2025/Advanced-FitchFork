import { expect, type Locator, type Page } from '@playwright/test';

export class SortModalPO {
  constructor(private readonly page: Page) {}

  get root(): Locator {
    return this.page.getByTestId('sort-modal').last();
  }

  async waitReady() {
    await expect(this.root).toBeVisible();
  }

  async addLevel() {
    await this.root.getByTestId('sort-add-level').click();
  }

  async setField(idx: number, fieldLabel: string | RegExp) {
    const select = this.root.getByTestId(`sort-field-${idx}`).first();
    await expect(select).toBeVisible();
    await select.click();

    const dropdown = this.page.locator('.ant-select-dropdown:visible').last();
    await expect(dropdown).toBeVisible();

    const rx = fieldLabel instanceof RegExp ? fieldLabel : new RegExp(`^${escapeRx(fieldLabel)}$`, 'i');
    const byRole = dropdown.getByRole('option', { name: rx }).first();
    if (await byRole.count()) {
      await byRole.click();
    } else {
      await dropdown.getByText(rx).first().click();
    }
  }

  /** order: 'ascend' | 'descend' */
  async setOrder(idx: number, order: 'ascend' | 'descend') {
    const seg = this.root.getByTestId(`sort-order-${idx}`).first();
    await expect(seg).toBeVisible();
    // Segmented supports clicking by label text
    const label = order === 'ascend' ? 'Asc' : 'Desc';
    await seg.getByText(label, { exact: true }).click();
  }

  async moveUp(idx: number) {
    await this.root.getByTestId(`sort-move-up-${idx}`).click();
  }
  async moveDown(idx: number) {
    await this.root.getByTestId(`sort-move-down-${idx}`).click();
  }
  async remove(idx: number) {
    await this.root.getByTestId(`sort-remove-${idx}`).click();
  }

  async apply() {
    await this.root.getByTestId('sort-apply').click();
  }

  async close() {
    await this.root.getByTestId('sort-close').click();
  }
}

function escapeRx(s: string) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
