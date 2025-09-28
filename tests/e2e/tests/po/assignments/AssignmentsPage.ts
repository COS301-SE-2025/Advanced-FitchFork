import { expect, type Page, type Response } from '@playwright/test';
import { EntityList } from '@po/common/EntityList';
import {
  BulkEditAssignmentsModal,
  CreateAssignmentModal,
  EditAssignmentModal,
  type AssignmentFormData,
} from './AssignmentModals';

export type AssignmentCreateInput = AssignmentFormData;
export type AssignmentEditInput = {
  name: string | RegExp;
  update: AssignmentFormData;
};

function escapeRe(s: string) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
function toExactPattern(name: string | RegExp): RegExp {
  return typeof name === 'string' ? new RegExp(`^${escapeRe(name)}$`) : name;
}

export class AssignmentsPage extends EntityList {
  readonly moduleId: number | string;

  constructor(page: Page, moduleId: number | string) {
    super(page, {
      route: `/modules/${moduleId}/assignments`,
      fetchRe: new RegExp(`/api/modules/${moduleId}/assignments\\b`),
      searchParam: 'query',
    });
    this.moduleId = moduleId;
  }

  /** Union of all entity container types */
  private containers() {
    return this.page
      .getByTestId('entity-row')
      .or(this.page.getByTestId('entity-list-item'))
      .or(this.page.getByTestId('entity-card'));
  }

  /** Find an entity container whose subtree contains a text node matching name exactly */
  entityByName(name: string | RegExp) {
    const pat = toExactPattern(name);
    return this.containers().filter({ has: this.page.getByText(pat) }).first();
  }

  /** Inner element that matches the exact name text */
  entityInnerByName(name: string | RegExp) {
    const pat = toExactPattern(name);
    return this.page.getByText(pat).first();
  }

  statusTagFor(name: string | RegExp) {
    return this.entityByName(name)
      .locator('.ant-tag')
      .filter({ hasText: /setup|ready|open|closed|archived/i })
      .first();
  }

  private async ensureControlCreateAvailable() {
    const emptyAdd = this.page.getByTestId('empty-add').first();
    if (await emptyAdd.isVisible().catch(() => false)) {
      await emptyAdd.click();
      return;
    }

    const ctrlCreate = this.page.getByTestId('control-action-create').first();
    if (await ctrlCreate.isVisible().catch(() => false)) {
      await ctrlCreate.click();
      return;
    }

    const fallback = this.page.getByRole('button', { name: /add assignment/i }).first();
    await expect(fallback).toBeVisible();
    await fallback.click();
  }

  async openCreateModal(): Promise<CreateAssignmentModal> {
    await this.ensureControlCreateAvailable();
    const modal = new CreateAssignmentModal(this.page);
    await modal.waitReady();
    return modal;
  }

  async createAssignmentUI(input: AssignmentCreateInput) {
    const modal = await this.openCreateModal();
    await modal.fill(input);

    const req: Promise<Response | null> = this.page
      .waitForResponse(
        r => r.request().method() === 'POST' && /\/api\/modules\/\d+\/assignments$/.test(r.url()),
      )
      .catch(() => null);

    await modal.submit();
    const resp = await req;
    if (resp) {
      expect(resp.status(), 'Create assignment should succeed').toBeGreaterThanOrEqual(200);
      expect(resp.status(), 'Create assignment should succeed').toBeLessThan(300);
    }

    await this.waitForNetworkIdle(350, 8000).catch(() => {});
    if (input.name) {
      await this.search(input.name);
      await expect(this.entityByName(input.name)).toBeVisible();
    } else {
      await this.waitForListLoadedOrEmpty();
    }
  }

  private async openEditAction(name: string | RegExp) {
    const scope = this.entityByName(name);
    const inline = scope.getByTestId('entity-action-edit').first();
    if (await inline.count()) {
      await inline.click();
      return;
    }

    await this.openRowDropdown(name);
    await this.clickRowDropdownAction('edit');
  }

  async openEditModal(name: string | RegExp): Promise<EditAssignmentModal> {
    await this.openEditAction(name);
    const modal = new EditAssignmentModal(this.page);
    await modal.waitReady();
    return modal;
  }

  async editAssignmentUI({ name, update }: AssignmentEditInput) {
    await this.search(typeof name === 'string' ? name : '');
    await expect(this.entityByName(name)).toBeVisible();

    const modal = await this.openEditModal(name);
    await modal.fill(update);

    const req: Promise<Response | null> = this.page
      .waitForResponse(r =>
        /^(PUT|PATCH)$/i.test(r.request().method()) &&
        /\/api\/modules\/\d+\/assignments\/\d+/.test(r.url()),
      )
      .catch(() => null);

    await modal.submit();
    const resp = await req;
    if (resp) {
      expect(resp.status(), 'Edit assignment should succeed').toBeGreaterThanOrEqual(200);
      expect(resp.status(), 'Edit assignment should succeed').toBeLessThan(300);
    }

    const targetName = update.name ?? name;
    if (typeof targetName === 'string') {
      await this.search(targetName);
    }
    await this.waitForNetworkIdle(350, 8000).catch(() => {});
    await expect(this.entityByName(targetName)).toBeVisible();
  }

  async openBulkEditModal(): Promise<BulkEditAssignmentsModal> {
    await this.clickBulkPrimary('bulk-edit');
    const modal = new BulkEditAssignmentsModal(this.page);
    await modal.waitReady();
    return modal;
  }

  async deleteAssignmentUI(name: string | RegExp) {
    await this.search(typeof name === 'string' ? name : '');
    await expect(this.entityByName(name)).toBeVisible();

    await this.openRowDropdown(name);
    await this.clickRowDropdownAction('delete');
    await this.confirmYes();
    await this.waitForNetworkIdle(350, 8000).catch(() => {});
    await expect(this.entityByName(name)).toHaveCount(0);
  }

  async openAssignmentUI(name: string | RegExp) {
    await this.openRowDropdown(name);
    await this.clickRowDropdownAction('open');
    await this.waitForNetworkIdle(350, 8000).catch(() => {});
  }

  async closeAssignmentUI(name: string | RegExp) {
    await this.openRowDropdown(name);
    await this.clickRowDropdownAction('close');
    await this.waitForNetworkIdle(350, 8000).catch(() => {});
  }
}
