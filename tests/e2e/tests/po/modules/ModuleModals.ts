// ModuleModals.ts
import { BaseModal } from '@po/common/BaseModal';

export class CreateModuleModal extends BaseModal {
  async fill(data: { code?: string; year?: number | string; description?: string; credits?: number | string }) {
    if (data.code !== undefined)        await this.fillByLabel(/module code/i, String(data.code));
    if (data.year !== undefined)        await this.fillByLabel(/year/i, String(data.year));
    if (data.description !== undefined) await this.fillByLabel(/description/i, data.description);
    if (data.credits !== undefined)     await this.fillByLabel(/credits/i, String(data.credits));
  }

  // 🔧 Validation path: resolve when modal closes OR any inline error appears
  async submit() {
    await this.waitReady();
    const btn = this.root.getByTestId('create-modal-submit').first()
      .or(this.root.getByRole('button', { name: /^create$/i }).first());
    await btn.click();
    await Promise.race([
      this.waitClosed(),
      this.waitAnyErrorVisible(4_000),
    ]);
  }

  async cancel() { await super.cancel('create-modal-cancel'); }

  codeError() { return this.errorUnder(/module code/i); }
  yearError() { return this.errorUnder(/year/i); }
  async waitForCodeError() { await this.waitForFieldError(/module code/i); }
}

export class EditModuleModal extends BaseModal {
  async fill(update: { code?: string; year?: number | string; description?: string; credits?: number | string }) {
    if (update.code !== undefined)        await this.fillByLabel(/module code/i, String(update.code));
    if (update.year !== undefined)        await this.fillByLabel(/year/i, String(update.year));
    if (update.description !== undefined) await this.fillByLabel(/description/i, update.description);
    if (update.credits !== undefined)     await this.fillByLabel(/credits/i, String(update.credits));
  }

  async save() {
    await this.waitReady();
    const btn = this.root.getByTestId('edit-modal-submit').first()
      .or(this.root.getByRole('button', { name: /^save$/i }).first());
    await btn.click();
    await Promise.race([
      this.waitClosed(),
      this.waitAnyErrorVisible(4_000),
    ]);
  }

  async cancel() { await super.cancel('edit-modal-cancel'); }
}
