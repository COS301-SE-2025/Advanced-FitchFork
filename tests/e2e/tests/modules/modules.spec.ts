import { test, expect } from '@fixtures';
import { ModulesPage, genModuleCode } from '@po/modules/ModulesPage';
import {
  createModuleAsAdmin,
  deleteModuleAsAdmin,
  type ModuleRecord,
} from '@helpers/modules';

test.describe('Modules / list page', () => {
  test('admin creates then deletes a module via the UI (table/list view)', async ({ as }) => {
    const page = await as('admin');
    const M = new ModulesPage(page);

    await M.goto();

    // Create via UI
    const code = genModuleCode();
    await M.createModuleUI({ code });

    // Toast: created
    await expect(page.getByText('Module created successfully')).toBeVisible({ timeout: 6000 });

    // Row present
    await expect(M.entityByModuleCode(code)).toBeVisible();

    // Delete via UI (cleanup)
    await M.deleteModuleUI(code);

    // Toast: deleted
    await expect(page.getByText('Module deleted successfully')).toBeVisible({ timeout: 6000 });

    // Gone
    await expect(M.entityByModuleCode(code)).toHaveCount(0);
  });

  test('rejects invalid module code on create', async ({ as }) => {
    const page = await as('admin');
    const M = new ModulesPage(page);

    await M.goto();

    const modal = await M.openCreateModal();

    // Not 3 uppercase letters + 3 digits
    const invalidCode = 'ab123';
    await modal.fill({
      code: invalidCode,
      year: new Date().getFullYear(),
      description: 'E2E invalid code',
      credits: 16,
    });

    await modal.submit();

    // Prefer global toast, fallback to field error if the UI blocks before POST
    let sawToast = false;
    try {
      await expect(
        page.getByText(/(module code.*format|format e\.g\.\s*COS212)/i)
      ).toBeVisible({ timeout: 6000 });
      sawToast = true;
    } catch {
      // fall back to field-level validation copy
    }

    if (!sawToast) {
      const fieldError = modal.codeError();
      await expect(fieldError).toBeVisible();
      await expect(fieldError).toHaveText(/(format|abc123|cos212|please enter)/i);
    }

    await modal.cancel();
    await expect(M.entityInnerByModuleCode(invalidCode)).toHaveCount(0);
  });

  test('rejects a past year on create', async ({ as }) => {
    const page = await as('admin');
    const M = new ModulesPage(page);

    await M.goto();

    const modal = await M.openCreateModal();

    const pastYear = new Date().getFullYear() - 1;
    const code = genModuleCode(); // valid format

    await modal.fill({
      code,
      year: pastYear,
      description: 'E2E past year',
      credits: 16,
    });

    await modal.submit();

    // Prefer global toast, fallback to field error
    let sawToast = false;
    try {
      await expect(
        page.getByText(/year must be (the )?current year or later/i)
      ).toBeVisible({ timeout: 6000 });
      sawToast = true;
    } catch {
      // look for inline field validation instead
    }

    if (!sawToast) {
      const fieldError = modal.yearError();
      await expect(fieldError).toBeVisible();
      await expect(fieldError).toHaveText(/(current year or later|please enter)/i);
    }

    await modal.cancel();
    await expect(M.entityInnerByModuleCode(code)).toHaveCount(0);
  });

  test.describe('with a pre-seeded module (API)', () => {
    let seeded: ModuleRecord;

    test.beforeEach(async ({ api }) => {
      const code = genModuleCode();
      seeded = await createModuleAsAdmin(api, {
        code,
        year: new Date().getFullYear(),
        description: 'Seeded by E2E',
        credits: 16,
      });
    });

    test.afterEach(async ({ api }) => {
      if (!seeded?.id) return;
      try {
        await deleteModuleAsAdmin(api, seeded.id);
      } catch {
        /* already deleted via UI (if a future test does that) */
      }
    });

    test('admin edits an existing module (table/list view)', async ({ as }) => {
      const page = await as('admin');
      const M = new ModulesPage(page);

      await M.goto();

      await M.search(seeded.code);
      await expect(M.entityInnerByModuleCode(seeded.code)).toBeVisible();

      await M.editModuleUI({
        code: seeded.code,
        update: { description: `Edited by E2E at ${Date.now()}`, credits: 24 },
      });

      // If your UI shows a toast here:
      await expect(
        page.getByText('Module updated successfully')
      ).toBeVisible({ timeout: 6000 }).catch(() => { /* ok if no toast on edit */ });

      await expect(page.getByText('Module updated successfully')).toBeVisible({ timeout: 6000 });

      await expect(M.entityInnerByModuleCode(seeded.code)).toBeVisible();
    });
  });
});
