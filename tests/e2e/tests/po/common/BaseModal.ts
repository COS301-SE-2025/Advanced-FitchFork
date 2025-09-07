import { expect, type Locator, type Page } from '@playwright/test';

export class BaseModal {
  constructor(protected readonly page: Page) {}

  /** Top-most visible AntD modal root */
  protected get root(): Locator {
    return this.page.locator('.ant-modal-root .ant-modal:visible').last();
  }

  async waitReady() {
    await expect(this.root).toBeVisible();
    await this.root
      .locator('input,textarea,[contenteditable="true"],button')
      .first()
      .waitFor({ state: 'visible' });
  }

  fieldByLabel(label: RegExp) {
    return this.root.getByLabel(label).first();
  }

  async fillByLabel(label: RegExp, value: string) {
    const el = this.fieldByLabel(label);
    await expect(el).toBeVisible();
    await el.fill(value);
  }

  /** Robust error node under the Form.Item for the given label (AntD v4/v5) */
  errorUnder(label: RegExp) {
    const item = this.fieldByLabel(label)
      .locator('xpath=ancestor::*[contains(@class,"ant-form-item")][1]');

    // Cover: explicit error node, role="alert" inside explain, and plain explain with text.
    return item
      .locator(
        '.ant-form-item-explain-error, .ant-form-item-explain [role="alert"], .ant-form-item-explain'
      )
      .filter({ hasText: /./ }) // only return if it actually has text
      .first();
  }

  /** Optional: wait until the Form.Item is showing an error for this field */
  async waitForFieldError(label: RegExp, timeout = 10_000) {
    const item = this.fieldByLabel(label)
      .locator('xpath=ancestor::*[contains(@class,"ant-form-item")][1]');

    // Race: either the explain/alert becomes visible or the item picks up the "error" status class.
    await Promise.race([
      this.errorUnder(label).waitFor({ state: 'visible', timeout }),
      item.locator(':scope.ant-form-item-status-error, :scope.ant-form-item-has-error')
          .waitFor({ state: 'attached', timeout }),
    ]);
  }

  async submit(preferredTestId: string, fallbackName: RegExp) {
    await this.waitReady();
    const byTid = this.root.getByTestId(preferredTestId).first();
    if (await byTid.count()) {
      await byTid.click();
      return;
    }
    await this.root.getByRole('button', { name: fallbackName }).first().click();
  }

  async cancel(preferredTestId?: string) {
    await this.waitReady();

    if (preferredTestId) {
      const byTid = this.root.getByTestId(preferredTestId).first();
      if (await byTid.count()) {
        await byTid.click();
        return;
      }
    }

    const footer = this.root.locator('.ant-modal-footer').first();
    if (await footer.count()) {
      const footerCancel = footer.getByRole('button', { name: /^cancel$/i }).first();
      if (await footerCancel.count()) {
        await footerCancel.click();
        return;
      }
    }

    const closeX = this.root.locator('.ant-modal-close').first();
    if (await closeX.count()) {
      await closeX.click();
      return;
    }

    await this.root.getByRole('button', { name: /^(cancel|close)$/i }).first().click();
  }
}
