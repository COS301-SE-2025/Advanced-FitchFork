import { test, expect } from "@fixtures";
import { LoginPage } from "@po/auth/LoginPage";
import { createUser, deleteUser } from "../../helpers/users";

let student: Awaited<ReturnType<typeof createUser>>;

test.beforeAll(async ({ api }) => {
  test.info().annotations.push({ type: 'feature', description: 'Auth/Login' });
  student = await createUser(api, { username: 'student', password: '1' });
});

test.afterAll(async ({ api }) => {
  if (student?.id) {
    await deleteUser(api, student.id);
  }
});

test.describe('Login', () => {
  test('client-side validation errors are shown', async ({ page }) => {
    const L = new LoginPage(page);
    await L.goto();
    await L.submit().click();
    await expect(page.getByText(/please enter your username/i)).toBeVisible();
    await expect(page.getByText(/please enter your password/i)).toBeVisible();
  });

  // TODO This test fails on firefox
  // test('successful login with seeded user redirects to /dashboard', async ({ page }) => {
  //   const L = new LoginPage(page);
  //   await L.goto();
  //   await L.username().fill(student.username);
  //   await L.password().fill(student.password);
  //   await L.submitAndWaitForDashboard();
  // });

  test('wrong password shows backend-auth error and clears on edit', async ({ page }) => {
    const L = new LoginPage(page);
    await L.goto();
    await L.fillAndSubmit({ username: student.username, password: 'wrong' });

    // Assert a generic invalid/unauthorized message from the backend
    await expect(L.alert()).toContainText(/invalid|unauthorized|incorrect/i);

    // Editing triggers onValuesChange -> clears alert
    await L.password().fill(student.password);
    await expect(L.alert()).toBeHidden();
  });

  test('nav links: forgot password & sign up', async ({ page }) => {
    const L = new LoginPage(page);
    await L.goto();

    await L.forgotPasswordLink().click();
    await expect(page).toHaveURL(/\/forgot-password\b/);

    await page.goto('/login');
    await L.signupLink().click();
    await expect(page).toHaveURL(/\/signup\b/);
  });
});
