import { expect, type Locator, type Page, type Response } from '@playwright/test';
import { EntityList } from '@po/common/EntityList';
import { CreateModuleModal, EditModuleModal } from './ModuleModals';

export function genModuleCode(): string {
  const L = () => String.fromCharCode(65 + Math.floor(Math.random() * 26));
  const D = () => Math.floor(100 + Math.random() * 900).toString();
  return `${L()}${L()}${L()}${D()}`; // ABC123
}

export class ModulesPage extends EntityList {
  constructor(page: Page) {
    super(page, {
      route: '/modules',
      fetchRe: /\/api\/modules\b/,
      searchParam: 'query',
    });
  }

  // ---------- stored "ABC123" -> displayed "ABC 123" (space or NBSP) ----------
  static toDisplay(code: string) {
    const raw = String(code).trim().replace(/\s+/g, '');
    return `${raw.slice(0, 3).toUpperCase()} ${raw.slice(3)}`;
  }
  private displayReFromStored(code: string) {
    const raw = String(code).trim().replace(/\s+/g, '');
    const letters = raw.slice(0, 3).toUpperCase();
    const digits  = raw.slice(3);
    // Allow normal or non-breaking space(s) between parts
    return new RegExp(`\\b${letters}[\\s\\u00A0]+${digits}\\b`, 'i');
  }

  // ---------- find entity by module code (stored) ----------
  entityByModuleCode(stored: string): Locator {
    return this.entityByText(this.displayReFromStored(stored));
  }
  entityInnerByModuleCode(stored: string): Locator {
    return this.entityInnerByText(this.displayReFromStored(stored));
  }

  // ---------- CREATE ----------
  async openCreateModal(): Promise<CreateModuleModal> {
    const emptyAdd = this.page.getByTestId('empty-add').first();
    if (await emptyAdd.isVisible().catch(() => false)) {
      await expect(emptyAdd).toBeEnabled();
      await emptyAdd.click();
    } else {
      const ctrlCreate = this.page.getByTestId('control-action-create').first();
      if (await ctrlCreate.isVisible().catch(() => false)) {
        await ctrlCreate.click();
      } else {
        const fallback = this.page.getByRole('button', { name: /add module/i }).first();
        await expect(fallback).toBeVisible();
        await expect(fallback).toBeEnabled();
        await fallback.click();
      }
    }

    const modal = new CreateModuleModal(this.page);
    await modal.waitReady();
    return modal;
  }

  async createModuleUI(input: {
    code: string;
    year?: number;
    description?: string;
    credits?: number;
  }) {
    const modal = await this.openCreateModal();

    await modal.fill({
      code: input.code,
      year: input.year ?? new Date().getFullYear(),
      description: input.description ?? 'Automated test module',
      credits: input.credits ?? 16,
    });

    const respP: Promise<Response> = this.page.waitForResponse(r => {
      if (r.request().method() !== 'POST') return false;
      return this.matchesFetch(r.url());
    });

    await modal.submit();
    const resp = await respP;
    expect(resp.status(), 'Create should succeed').toBeGreaterThanOrEqual(200);
    expect(resp.status(), 'Create should succeed').toBeLessThan(300);

    // allow refetch to complete, then search and assert
    const refetch = this.listFetch.catch(() => null);
    await Promise.race([refetch, this.waitForNetworkIdle(350, 8000)]).catch(() => {});
    await this.waitForNetworkIdle(200, 4000);

    await this.search(input.code);
    await expect(this.entityByModuleCode(input.code)).toBeVisible();
  }

  // ---------- EDIT ----------
  async openEditFor(storedCode: string): Promise<EditModuleModal> {
    await this.clickRowPrimaryAction(this.displayReFromStored(storedCode), 'edit');
    const modal = new EditModuleModal(this.page);
    await modal.waitReady();
    return modal;
  }

  async editModuleUI(params: {
    code: string;
    update: Partial<{ code: string; year: number; description: string; credits: number }>;
  }) {
    await this.search(params.code);
    await expect(this.entityByModuleCode(params.code)).toBeVisible();

    const modal = await this.openEditFor(params.code);
    await modal.fill(params.update);

    const respP = this.page.waitForResponse(r => {
      if (!/\/api\/modules\/\d+\b/.test(r.url())) return false;
      const m = r.request().method();
      return m === 'PUT' || m === 'PATCH';
    });

    await modal.save();
    const r = await respP;
    expect(r.status(), 'Edit should succeed').toBeGreaterThanOrEqual(200);
    expect(r.status(), 'Edit should succeed').toBeLessThan(300);

    const finalCode = params.update.code ?? params.code;
    await this.search(finalCode);
    await expect(this.entityByModuleCode(finalCode)).toBeVisible();
  }

  // ---------- DELETE ----------
  async deleteModuleUI(storedCode: string) {
    await this.search(storedCode);
    const text = this.displayReFromStored(storedCode);
    
    await this.openRowDropdown(text);
    await this.clickRowDropdownAction('delete');

    const refetch = this.listFetch.catch(() => null);
    await this.confirmYes();
    await Promise.race([refetch, this.page.waitForTimeout(400)]);

    await expect(this.entityByModuleCode(storedCode)).toHaveCount(0);
  }
}
