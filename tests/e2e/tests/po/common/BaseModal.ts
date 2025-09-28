// BaseModal.ts
import { expect, type Locator, type Page } from '@playwright/test';

export class BaseModal {
  constructor(protected readonly page: Page) {}

  protected get root(): Locator {
    return this.page.locator('.ant-modal-root .ant-modal:visible').last();
  }
  private get wrap(): Locator {
    return this.page.locator('.ant-modal-root .ant-modal-wrap').last();
  }
  private get mask(): Locator {
    return this.page.locator('.ant-modal-root .ant-modal-mask').last();
  }

  async waitReady() {
    await expect(this.root).toBeVisible();
    await this.root.locator('input,textarea,[contenteditable="true"],button')
      .first().waitFor({ state: 'visible' });
  }

  protected async waitClosed() {
    try {
      await this.page.locator('.ant-modal-root .ant-modal:visible').last()
        .waitFor({ state: 'hidden', timeout: 10_000 }).catch(() => {});
      await this.page.locator('.ant-modal-root .ant-modal-wrap:visible')
        .waitFor({ state: 'hidden', timeout: 3_000 }).catch(() => {});
      await this.page.locator('.ant-modal-root .ant-modal-mask:visible')
        .waitFor({ state: 'hidden', timeout: 3_000 }).catch(() => {});
      if (!this.page.isClosed()) await this.page.waitForTimeout(50).catch(() => {});
    } catch {}
  }

  // ----------- helpers -----------
  /** Prefer locating the Form.Item via its label text (AntD doesn’t tie <label for> reliably). */
  protected formItemByLabel(label: RegExp) {
    return this.root
      .locator('.ant-form-item-label')
      .filter({ hasText: label })
      .locator('xpath=ancestor::*[contains(@class,"ant-form-item")][1]')
      .first();
  }

  /** For inputs where label lookup works, still useful sometimes. */
  fieldByLabel(label: RegExp) {
    return this.root.getByLabel(label).first();
  }

  async selectByLabel(label: RegExp, option: string | RegExp) {
    const item = this.formItemByLabel(label);
    await expect(item).toBeVisible();
    const trigger = item.locator('[role="combobox"], .ant-select-selector').first();
    await trigger.click();
    const dropdown = this.page.locator('.ant-select-dropdown:visible').last();
    await expect(dropdown).toBeVisible();
    const rx = option instanceof RegExp ? option : new RegExp(`^${escapeRegExp(option)}$`, 'i');
    const byRole = dropdown.getByRole('option', { name: rx }).first();
    if (await byRole.count()) { await byRole.click(); return; }
    await dropdown.getByText(rx).first().click();
  }

  async fillByLabel(label: RegExp, value: string) {
    // try field first; fall back to the Form.Item’s input
    const byLabel = this.fieldByLabel(label);
    if (await byLabel.count()) {
      await expect(byLabel).toBeVisible();
      await byLabel.fill(value);
      return;
    }
    const item = this.formItemByLabel(label);
    const input = item.locator('input,textarea,[contenteditable="true"]').first();
    await expect(input).toBeVisible();
    await input.fill(value);
  }

  /** Inline error under a specific Form.Item */
  errorUnder(label: RegExp) {
    const item = this.formItemByLabel(label);
    return item.locator(
      '.ant-form-item-explain-error, .ant-form-item-explain [role="alert"], .ant-form-item-explain'
    ).filter({ hasText: /./ }).first();
  }

  async waitForFieldError(label: RegExp, timeout = 10_000) {
    const item = this.formItemByLabel(label);
    await Promise.race([
      this.errorUnder(label).waitFor({ state: 'visible', timeout }),
      item.locator(':scope.ant-form-item-status-error, :scope.ant-form-item-has-error')
          .waitFor({ state: 'attached', timeout }),
    ]).catch(() => {});
  }

  // Global “any error appeared” detector (kept as-is)
  protected anyExplain(): Locator {
    return this.root
      .locator('.ant-form-item-explain-error, .ant-form-item-explain [role="alert"], .ant-form-item-explain')
      .filter({ hasText: /./ });
  }
  protected async waitAnyErrorVisible(timeout = 4_000) {
    try {
      await this.anyExplain().first().waitFor({ state: 'visible', timeout });
    } catch {}
  }

  async submit(preferredTestId: string, fallbackName: RegExp) {
    await this.waitReady();
    const byTid = this.root.getByTestId(preferredTestId).first();
    if (await byTid.count()) { await byTid.click(); await this.waitClosed(); return; }
    await this.root.getByRole('button', { name: fallbackName }).first().click();
    await this.waitClosed();
  }

  /** 🔙 Restored: used by tests and subclasses */
  async cancel(preferredTestId?: string) {
    await this.waitReady();

    if (preferredTestId) {
      const byTid = this.root.getByTestId(preferredTestId).first();
      if (await byTid.count()) { await byTid.click(); await this.waitClosed(); return; }
    }

    const footer = this.root.locator('.ant-modal-footer').first();
    if (await footer.count()) {
      const btn = footer.getByRole('button', { name: /^cancel$/i }).first();
      if (await btn.count()) { await btn.click(); await this.waitClosed(); return; }
    }

    const closeX = this.root.locator('.ant-modal-close').first();
    if (await closeX.count()) { await closeX.click(); await this.waitClosed(); return; }

    await this.root.getByRole('button', { name: /^(cancel|close)$/i }).first().click();
    await this.waitClosed();
  }
}

function escapeRegExp(s: string) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
