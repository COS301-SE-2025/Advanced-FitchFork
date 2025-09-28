import { expect, type Locator, type Page, type Response } from '@playwright/test';

export type EntityListOptions = {
  route: string;
  fetchRe: RegExp;
  searchParam?: string;
};

export class EntityList {
  readonly page: Page;
  readonly opts: Required<EntityListOptions>;

  constructor(page: Page, opts: EntityListOptions) {
    this.page = page;
    this.opts = { searchParam: 'query', ...opts } as Required<EntityListOptions>;
  }

  // ---------------- navigation ----------------
  async goto() {
    await this.page.goto(this.opts.route);
    await this.waitForListLoadedOrEmpty();
  }

  // ---------------- core waits ----------------
  protected async waitForNetworkIdle(idleMs = 300, timeout = 6000) {
    const start = Date.now();
    let lastSeen = Date.now();
    let inFlight = 0;

    const isListReq = (url: string, method: string) =>
      method === 'GET' && this.opts.fetchRe.test(url);

    const onReq = (req: any) => {
      if (isListReq(req.url(), req.method())) { inFlight++; lastSeen = Date.now(); }
    };
    const onFin = (req: any) => {
      if (isListReq(req.url(), req.method())) { inFlight = Math.max(0, inFlight - 1); lastSeen = Date.now(); }
    };

    this.page.on('request', onReq);
    this.page.on('requestfinished', onFin);
    this.page.on('requestfailed', onFin);

    try {
      while (Date.now() - start < timeout) {
        if (inFlight === 0 && Date.now() - lastSeen >= idleMs) return;
        await this.page.waitForTimeout(50);
      }
      throw new Error(`EntityList.waitForNetworkIdle timeout after ${timeout}ms`);
    } finally {
      this.page.off('request', onReq);
      this.page.off('requestfinished', onFin);
      this.page.off('requestfailed', onFin);
    }
  }

  protected get listFetch(): Promise<Response> {
    return this.page.waitForResponse(
      r => r.request().method() === 'GET' && this.opts.fetchRe.test(r.url()),
    );
  }

  async waitForListLoadedOrEmpty() {
    const firstVisible = (sel: string) =>
      this.page.locator(sel).first().waitFor({ state: 'visible' }).catch(() => {});

    await Promise.race([
      this.listFetch.catch(() => {}),

      // Your containers WITH at least one item inside (no AntD fallbacks)
      firstVisible('[data-testid="entity-table"] [data-testid="entity-row"]'),
      firstVisible('[data-testid="entity-list"]  [data-testid="entity-list-item"]'),
      firstVisible('[data-testid="entity-grid"]  [data-testid="entity-card"]'),

      // Or any item directly
      firstVisible('[data-testid="entity-row"]'),
      firstVisible('[data-testid="entity-list-item"]'),
      firstVisible('[data-testid="entity-card"]'),

      // Empty state or search box
      firstVisible('.ant-empty'),
      firstVisible('[data-testid="entity-search"]'),

      this.page.waitForTimeout(200),
    ]);
  }

  // ---------------- helpers (TEXT-ONLY; prefers your testids) ----------------
  private ensureRegex(textOrRe: string | RegExp): RegExp {
    if (textOrRe instanceof RegExp) return textOrRe;
    // Make spaces match normal or NBSP
    const escaped = escapeRegExp(textOrRe).replace(/\s+/g, '[\\s\\u00A0]+');
    // Be less opinionated about word boundaries across nodes
    return new RegExp(`${escaped}`, 'i');
  }

  /** Entity container that contains the given text (row/list-item/card only). */
  entityByText(textOrRe: string | RegExp): Locator {
    const rx = this.ensureRegex(textOrRe);
    const txt = this.page.getByText(rx);

    const row      = this.page.getByTestId('entity-row').filter({ has: txt });
    const listItem = this.page.getByTestId('entity-list-item').filter({ has: txt });
    const card     = this.page.getByTestId('entity-card').filter({ has: txt });

    return row.or(listItem).or(card).first();
  }

  /** Inner element that has the text inside that entity. */
  entityInnerByText(textOrRe: string | RegExp): Locator {
    const rx = this.ensureRegex(textOrRe);
    return this.entityByText(rx).getByText(rx).first();
  }

  // ---------------- view mode ----------------
  async ensureTableView() {
    if ((await this.page.getByTestId('entity-table').count()) > 0) return;
    const toggle = this.page.getByTestId('view-toggle-table').first();
    if (await toggle.count()) await toggle.click();
    await this.page.getByTestId('entity-table').first().waitFor({ state: 'visible' }).catch(() => {});
  }

  async ensureGridView() {
    if ((await this.page.getByTestId('entity-grid').count()) > 0) return;
    const toggle = this.page.getByTestId('view-toggle-grid').first();
    if (await toggle.count()) await toggle.click();
    await Promise.race([
      this.page.getByTestId('entity-grid').first().waitFor({ state: 'visible' }),
      this.page.getByTestId('entity-list').first().waitFor({ state: 'visible' }),
      this.page.waitForTimeout(200),
    ]).catch(() => {});
    await expect(this.page.getByTestId('entity-table')).toHaveCount(0);
  }

  // ---------------- search ----------------
  async search(term: string) {
    const { searchParam } = this.opts;
    const searchBox = this.page.getByTestId('entity-search').first();

    await searchBox.click();
    await searchBox.fill(term);

    const expectQueryResponse = this.page
      .waitForResponse(r => {
        if (r.request().method() !== 'GET') return false;
        if (!this.opts.fetchRe.test(r.url())) return false;
        try {
          const u = new URL(r.url());
          const q = u.searchParams.get(searchParam) ?? '';
          return q === term && r.status() >= 200 && r.status() < 300;
        } catch { return false; }
      })
      .catch(() => null);

    await Promise.race([expectQueryResponse, this.waitForNetworkIdle(350, 8000)]);
    await this.waitForListLoadedOrEmpty();
  }

  // ---------------- control-bar actions ----------------
  async clickControlPrimary(key: string) {
    const btn = this.page.getByTestId(`control-action-${key}`).first();
    await expect(btn, `Primary control [control-action-${key}] should be visible`).toBeVisible();
    await btn.click();
  }
  async openControlActionsDropdown() {
    const trigger = this.page.getByTestId('control-action-dropdown').first();
    await expect(trigger).toBeVisible();
    await trigger.click();
    await expect(this.page.locator('.ant-dropdown:visible').first()).toBeVisible();
  }
  async clickControlDropdownAction(key: string) {
    const scope = this.page.locator('.ant-dropdown:visible').first();
    const item = scope.getByTestId(`control-action-${key}`).first();
    await expect(item).toBeVisible();
    await item.click();
  }

  // ---------------- bulk actions ----------------
  async clickBulkPrimary(key: string) {
    const btn = this.page.getByTestId(`bulk-action-${key}`).first();
    await expect(btn, `Bulk action [bulk-action-${key}] should be visible`).toBeVisible();
    await btn.click();
  }
  async openBulkActionsDropdown() {
    const trigger = this.page.getByTestId('bulk-action-dropdown').first();
    await expect(trigger).toBeVisible();
    await trigger.click();
    await expect(this.page.locator('.ant-dropdown:visible').first()).toBeVisible();
  }
  async clickBulkDropdownAction(key: string) {
    const scope = this.page.locator('.ant-dropdown:visible').first();
    const item = scope.getByTestId(`bulk-action-${key}`).first();
    await expect(item).toBeVisible();
    await item.click();
  }

  // ---------------- per-entity actions (text-scoped) ----------------
  async clickRowPrimaryAction(rowText: string | RegExp, actionKey: string) {
    const scope = this.entityByText(rowText);
    const byTid = scope.getByTestId(`entity-action-${actionKey}`).first();
    if (await byTid.count()) { await expect(byTid).toBeVisible(); await byTid.click(); return; }

    // minimal fallback via icon
    const icon = scope.getByRole('img', { name: new RegExp(actionKey, 'i') }).first();
    if (await icon.count()) {
      const btn = icon.locator('xpath=ancestor::button[1]').first();
      await expect(btn).toBeVisible();
      await btn.click();
      return;
    }

    throw new Error(`Could not find action "${actionKey}" in entity containing "${rowText}"`);
  }

  async openRowDropdown(rowText: string | RegExp) {
    const scope = this.entityByText(rowText);
    const trigger = scope.getByTestId('entity-action-dropdown').first();
    await expect(trigger, 'Entity actions dropdown trigger should exist').toBeVisible();
    await trigger.click();
    await expect(this.page.locator('.ant-dropdown:visible').first()).toBeVisible();
  }
  async clickRowDropdownAction(actionKey: string) {
    const scope = this.page.locator('.ant-dropdown:visible').first();
    const item = scope.getByTestId(`entity-action-${actionKey}`).first();
    await expect(item).toBeVisible();
    await item.click();
  }

  // ---------------- row selection ----------------
  async setRowSelection(rowText: string | RegExp, selected = true) {
    const scope = this.entityByText(rowText);
    const checkbox = scope.getByRole('checkbox').first();
    await expect(checkbox, 'Row selection checkbox should exist').toBeAttached();

    const isChecked = await checkbox.isChecked().catch(() => false);
    if (isChecked === selected) return;

    if (selected) {
      await checkbox.check({ force: true });
    } else {
      await checkbox.uncheck({ force: true });
    }
  }

  async clearAllSelection() {
    const checkedBoxes = this.page.getByRole('checkbox', { checked: true });
    const total = await checkedBoxes.count();
    for (let i = 0; i < total; i++) {
      const box = checkedBoxes.nth(i);
      await box.uncheck({ force: true }).catch(() => {});
    }
  }

  // ---------------- modal helpers ----------------
  protected get visibleModal() {
    return this.page.locator('.ant-modal-root .ant-modal:visible').last();
  }
  protected async waitForModalReady() {
    const modal = this.visibleModal;
    await expect(modal, 'Visible modal should appear').toBeVisible();
    await modal.locator('input,textarea,[contenteditable="true"],button').first()
      .waitFor({ state: 'visible' });
  }

  // ---------------- confirm helpers ----------------
  async confirmYes() { const yes = this.page.getByTestId('confirm-yes').first(); await expect(yes).toBeVisible(); await yes.click(); }
  async confirmNo()  { const no  = this.page.getByTestId('confirm-no').first();  await expect(no ).toBeVisible(); await no .click(); }

  // ---------------- pagination helpers ----------------
  async gridPrev() { await this.page.getByTestId('grid-previous').click(); await this.waitForNetworkIdle(); }
  async gridNext() { await this.page.getByTestId('grid-next').click(); await this.waitForNetworkIdle(); }
  async listPrev() { await this.page.getByTestId('list-previous').click(); await this.waitForNetworkIdle(); }
  async listNext() { await this.page.getByTestId('list-next').click(); await this.waitForNetworkIdle(); }
}

/* utils */
function escapeRegExp(s: string) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
