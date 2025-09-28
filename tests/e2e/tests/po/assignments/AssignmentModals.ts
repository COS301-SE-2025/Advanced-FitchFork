import { BaseModal } from '@po/common/BaseModal';

export type AssignmentFormData = {
  name?: string;
  description?: string;
  assignment_type?: string;
  available_from?: string;
  due_date?: string;
};

export class AssignmentFormModal extends BaseModal {
  async fill(data: AssignmentFormData) {
    if (data.name !== undefined) {
      await this.fillByLabel(/^name$/i, data.name);
    }
    if (data.description !== undefined) {
      await this.fillByLabel(/description/i, data.description);
    }
    if (data.assignment_type !== undefined) {
      await this.selectByLabel(/type/i, data.assignment_type);
    }
    if (data.available_from !== undefined) {
      await this.fillByLabel(/available from/i, data.available_from);
    }
    if (data.due_date !== undefined) {
      await this.fillByLabel(/due date/i, data.due_date);
    }
  }

  protected async submitByLabel(label: RegExp) {
    await this.waitReady();
    await this.root.getByRole('button', { name: label }).first().click();
    await this.waitClosed();
  }

  async cancel() {
    await super.cancel();
  }
}

export class CreateAssignmentModal extends AssignmentFormModal {
  async submit() {
    await this.submitByLabel(/^create$/i);
  }
}

export class EditAssignmentModal extends AssignmentFormModal {
  async submit() {
    await this.submitByLabel(/^(save|update)$/i);
  }
}

export type BulkEditAssignmentData = {
  status?: string;
  available_from?: string;
  due_date?: string;
};

export class BulkEditAssignmentsModal extends BaseModal {
  async fill(data: BulkEditAssignmentData) {
    if (data.status !== undefined) {
      await this.selectByLabel(/status/i, data.status);
    }
    if (data.available_from !== undefined) {
      await this.fillByLabel(/available from/i, data.available_from);
    }
    if (data.due_date !== undefined) {
      await this.fillByLabel(/due date/i, data.due_date);
    }
  }

  async submit() {
    await this.waitReady();
    await this.root.getByRole('button', { name: /^(update|save)$/i }).first().click();
    await this.waitClosed();
  }
  async cancel() {
    await super.cancel();
  }
}
