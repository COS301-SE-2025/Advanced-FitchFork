import { BaseModal } from '@po/common/BaseModal';

export class CreateModuleModal extends BaseModal {
  async fill(data: { code?: string; year?: number | string; description?: string; credits?: number | string }) {
    if (data.code !== undefined)        await this.fillByLabel(/module code/i, String(data.code));
    if (data.year !== undefined)        await this.fillByLabel(/year/i, String(data.year));
    if (data.description !== undefined) await this.fillByLabel(/description/i, data.description);
    if (data.credits !== undefined)     await this.fillByLabel(/credits/i, String(data.credits));
  }
  async submit() { await super.submit('create-modal-submit', /^create$/i); }
  async cancel() { await super.cancel('create-modal-cancel'); }

  codeError() { return this.errorUnder(/module code/i); }
  yearError() { return this.errorUnder(/year/i); }

  // Convenience if you want it in tests:
  async waitForCodeError() { await this.waitForFieldError(/module code/i); }
}

export class EditModuleModal extends BaseModal {
  async fill(update: { code?: string; year?: number | string; description?: string; credits?: number | string }) {
    if (update.code !== undefined)        await this.fillByLabel(/module code/i, String(update.code));
    if (update.year !== undefined)        await this.fillByLabel(/year/i, String(update.year));
    if (update.description !== undefined) await this.fillByLabel(/description/i, update.description);
    if (update.credits !== undefined)     await this.fillByLabel(/credits/i, String(update.credits));
  }
  async save()   { await super.submit('edit-modal-submit',  /^save$/i); }
  async cancel() { await super.cancel('edit-modal-cancel'); }
}
