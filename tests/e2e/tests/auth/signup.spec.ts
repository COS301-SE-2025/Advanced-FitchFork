import { test, expect } from "@fixtures";
import { SignupPage } from "@po/auth/SignupPage";


function randomDigits(n: number) {
  return Array.from({ length: n }, () => Math.floor(Math.random() * 10)).join('');
}
function newUsername() {
  return `user-${Date.now()}-${randomDigits(4)}`;
}
function newEmail(prefix = 'user') {
  return `${prefix}-${Date.now()}-${randomDigits(4)}@up.ac.za`;
}

test.beforeAll(() => {
  test.info().annotations.push({ type: 'feature', description: 'Auth/Signup' });
});

test.describe('Signup', () => {
  test('client-side validation errors are shown', async ({ page }) => {
    test.info().annotations.push({ type: 'severity', description: 'medium' });

    const S = new SignupPage(page);
    await test.step('Open the signup page', async () => {
      await S.goto();
    });

    await test.step('Submit empty form', async () => {
      await S.submit().click();
      await expect(page.getByText(/please enter your username/i)).toBeVisible();
      await expect(page.getByText(/please enter your email/i)).toBeVisible();
      await expect(page.getByText(/please enter your password/i)).toBeVisible();
      await expect(page.getByText(/please confirm your password/i)).toBeVisible();
    });

    await test.step('Bad email + weak password + mismatch', async () => {
      await S.username().fill(newUsername());
      await S.email().fill('not-an-email');
      await S.password().fill('short');
      await S.confirm().fill('different');

      // trigger validation
      await page.keyboard.press('Tab');

      await expect(page.getByText(/enter a valid email address/i)).toBeVisible();
      await expect(page.getByText(/at least 8 characters/i)).toBeVisible();

      await S.password().fill('Password1');
      await S.confirm().fill('Password2');
      await S.submit().click();
      await expect(page.getByText(/passwords do not match/i)).toBeVisible();
    });
  });

  test('successful signup redirects to /login (201 Created)', async ({ page }) => {
    const S = new SignupPage(page);
    await S.goto();

    const username = newUsername();
    const email = newEmail('ok');
    await S.fillAndSubmit({
      username,
      email,
      password: 'Password123',
      confirm: 'Password123',
    });

    await expect(page).toHaveURL(/\/login\b/);
  });

  test('email duplicate surfaces 409 conflict message', async ({ page }) => {
    const S = new SignupPage(page);

    // 1) Create a user with a fresh email
    await S.goto();
    const firstUsername = newUsername();
    const dupEmail = newEmail('dupe');

    await S.fillAndSubmit({
      username: firstUsername,
      email: dupEmail,
      password: 'Password123',
      confirm: 'Password123',
    });
    await expect(page).toHaveURL(/\/login\b/);

    // 2) Try again with SAME email but different username -> should hit email uniqueness 409
    await page.goto('/signup');
    await S.fillAndSubmit({
      username: newUsername(),
      email: dupEmail, // same email
      password: 'Password123',
      confirm: 'Password123',
    });

    await expect(S.errorAlert()).toContainText(/already exists/i);
  });

  test('username duplicate surfaces 409 conflict message', async ({ page }) => {
    const S = new SignupPage(page);

    // 1) Create a user with a fresh username
    await S.goto();
    const dupUsername = newUsername();

    await S.fillAndSubmit({
      username: dupUsername,
      email: newEmail('user1'),
      password: 'Password123',
      confirm: 'Password123',
    });
    await expect(page).toHaveURL(/\/login\b/);

    // 2) Try again with SAME username but different email -> should hit username uniqueness 409
    await page.goto('/signup');
    await S.fillAndSubmit({
      username: dupUsername,          // same username
      email: newEmail('user2'),       // different email
      password: 'Password123',
      confirm: 'Password123',
    });

    // Backend message might say "A user with this student number already exists"
    // or similar — keep assertion generic:
    await expect(S.errorAlert()).toContainText(/already exists/i);
  });
});
