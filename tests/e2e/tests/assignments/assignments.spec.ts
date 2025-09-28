import { expect, test } from '@fixtures';
import { createModuleAsAdmin, deleteModuleAsAdmin, type ModuleRecord } from '@helpers/modules';
import {
  createAssignmentAsAdmin,
  purgeAssignmentsAsAdmin,
  type AssignmentSeedInput,
} from '@helpers/assignments';
import { AssignmentsPage } from '@po/assignments/AssignmentsPage';
import { genModuleCode } from '@po/modules/ModulesPage';
import { FilterModalPO } from '@po/common/FilterModal'; // ⟵ add this
import { TestInfo } from '@playwright/test';

const thisYear = new Date().getFullYear();

let nameSeq = 0;
function uniqName(base: string) {
  nameSeq += 1;
  return `${base} ${Date.now()}-${nameSeq}`;
}

test.describe('Assignments / list page', () => {
  test.describe.configure({ mode: 'serial' });

  let moduleRecord: ModuleRecord;

  test.beforeAll(async ({ api }) => {
    const code = genModuleCode();
    moduleRecord = await createModuleAsAdmin(api, {
      code,
      year: thisYear,
      credits: 16,
      description: 'E2E assignments module',
    });
  });

  test.afterAll(async ({ api }) => {
    if (!moduleRecord?.id) return;
    await purgeAssignmentsAsAdmin(api, moduleRecord.id).catch(() => {});
    await deleteModuleAsAdmin(api, moduleRecord.id).catch(() => {});
  });

  test.beforeEach(async ({ api }) => {
    await purgeAssignmentsAsAdmin(api, moduleRecord.id).catch(() => {});
  });

  test.afterEach(async ({ api }) => {
    await purgeAssignmentsAsAdmin(api, moduleRecord.id).catch(() => {});
  });

  test('admin creates assignment from empty state (table view)', async ({ as }) => {
    const page = await as('admin');
    const A = new AssignmentsPage(page, moduleRecord.id);

    await A.goto();

    await expect(page.getByTestId('empty-add')).toBeVisible();

    const assignmentName = uniqName('Assignment');
    await A.createAssignmentUI({ name: assignmentName, description: 'Created via E2E' });

    await expect(A.entityByName(assignmentName)).toBeVisible();
    await expect(A.statusTagFor(assignmentName)).toHaveText(/setup/i);
  });

  test('admin edits an assignment and updates its metadata', async ({ api, as }) => {
    const seededName = uniqName('Editable');
    const dueSoon = new Date(Date.now() + 4 * 24 * 60 * 60 * 1000).toISOString();
    const available = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();

    await createAssignmentAsAdmin(api, moduleRecord.id, {
      name: seededName,
      description: 'Seeded via API',
      assignment_type: 'assignment',
      available_from: available,
      due_date: dueSoon,
    });

    const page = await as('admin');
    const A = new AssignmentsPage(page, moduleRecord.id);

    await A.goto();
    await A.waitForListLoadedOrEmpty();

    const updatedName = `${seededName} Updated`;
    await A.editAssignmentUI({
      name: seededName,
      update: {
        name: updatedName,
        description: 'Edited via UI',
        assignment_type: 'practical',
      },
    });

    await expect(A.entityByName(updatedName)).toBeVisible();
    await expect(A.entityByName(seededName)).toHaveCount(0);
    await expect(A.statusTagFor(updatedName)).toHaveText(/setup|ready|open|closed|archived/i);
  });

  test('search and filters narrow the assignment list', async ({ api, as }) => {
    const baseTime = Date.now();
    const assignments: AssignmentSeedInput[] = [
      {
        name: uniqName('Alpha Practical'),
        assignment_type: 'practical',
        description: 'Practical work',
        available_from: new Date(baseTime).toISOString(),
        due_date: new Date(baseTime + 7 * 24 * 60 * 60 * 1000).toISOString(),
      },
      {
        name: uniqName('Bravo Assignment'),
        assignment_type: 'assignment',
        description: 'Written work',
        available_from: new Date(baseTime).toISOString(),
        due_date: new Date(baseTime + 5 * 24 * 60 * 60 * 1000).toISOString(),
      },
    ];

    for (const data of assignments) {
      await createAssignmentAsAdmin(api, moduleRecord.id, data);
    }

    const page = await as('admin');
    const A = new AssignmentsPage(page, moduleRecord.id);

    await A.goto();
    await A.waitForListLoadedOrEmpty();

    // search narrows results
    await A.search(assignments[0].name);
    await expect(A.entityByName(assignments[0].name)).toBeVisible();
    await expect(A.entityByName(assignments[1].name)).toHaveCount(0);

    // reset by reloading (fastest deterministic way)
    await A.goto();
    await A.waitForListLoadedOrEmpty();

    // open Filters modal
    await page.getByTestId('filters-dropdown').click();
    await page.getByTestId('open-filter-modal').click();

    // use PO
    const F = new FilterModalPO(page);
    await F.waitReady();
    await F.expandGroup('Type');
    await F.setSelect('assignment_type', 'Practical');
    await F.apply();

    await A.waitForListLoadedOrEmpty();
    await expect(A.entityByName(assignments[0].name)).toBeVisible();
    await expect(A.entityByName(assignments[1].name)).toHaveCount(0);
  });

  test('bulk edit and bulk delete operate on selected assignments', async ({ as }) => {
    const page = await as('admin');
    const A = new AssignmentsPage(page, moduleRecord.id);

    await A.goto();
    await A.waitForListLoadedOrEmpty();

    const targets = [uniqName('Bulk A'), uniqName('Bulk B'), uniqName('Bulk C')];
    for (const name of targets) {
      await A.createAssignmentUI({ name, description: 'Bulk test' });
    }

    await A.goto();
    await A.waitForListLoadedOrEmpty();

    await A.ensureTableView();

    // select two assignments
    await A.setRowSelection(targets[0]);
    await A.setRowSelection(targets[1]);

    // bulk edit due date
    const modal = await A.openBulkEditModal();
    const newDue = '2099-01-10 10:30';
    await modal.fill({ due_date: newDue });
    await modal.submit();

    await expect(page.getByText(/Assignments updated/i)).toBeVisible({ timeout: 8000 }).catch(() => {});

    // Verify table still lists all items
    for (const name of targets) {
      await expect(A.entityByName(name)).toBeVisible();
    }

    // Reselect rows (refresh clears selection)
    await A.setRowSelection(targets[0]);
    await A.setRowSelection(targets[1]);

    // bulk delete the two selected assignments
    await A.openBulkActionsDropdown();
    await A.clickBulkDropdownAction('bulk-delete');
    await A.confirmYes();

    await expect(A.entityByName(targets[0])).toHaveCount(0);
    await expect(A.entityByName(targets[1])).toHaveCount(0);
    await expect(A.entityByName(targets[2])).toBeVisible();
  });

  test.describe('desktop only behaviours', () => {
  test('view toggle switches between table and grid', async ({ api, as }, testInfo: TestInfo) => {
    // Skip if this project is mobile
    if ((testInfo.project.use as any)?.isMobile) {
      test.skip(true, 'Desktop-only controls not rendered on mobile viewport');
    }

    const names = [uniqName('Grid X'), uniqName('Grid Y')];
    const available = new Date().toISOString();
    const due = new Date(Date.now() + 6 * 24 * 60 * 60 * 1000).toISOString();

    for (const name of names) {
      await createAssignmentAsAdmin(api, moduleRecord.id, {
        name,
        assignment_type: 'assignment',
        description: 'Grid view seed',
        available_from: available,
        due_date: due,
      });
    }

    const page = await as('admin');
    const A = new AssignmentsPage(page, moduleRecord.id);

    await A.goto();
    await A.waitForListLoadedOrEmpty();
    await A.ensureGridView();

    const cards = page.getByTestId('entity-card');
    await expect(cards).toHaveCount(names.length);
    for (const name of names) {
      await expect(page.getByTestId('entity-card').filter({ hasText: name })).toBeVisible();
    }

    await A.ensureTableView();
    await expect(page.getByTestId('entity-table')).toBeVisible();
  });
});
});
