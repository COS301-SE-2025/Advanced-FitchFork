# k6 Performance Testing Suite

This directory contains the performance testing suite for the application, using [k6](https://k6.io/).

## Directory Structure

-   `/scenarios`: Contains the individual test scripts, each simulating a specific user journey or scenario.
-   `/shared`: Contains shared code used by the test scripts, such as authentication logic (`auth.js`) and configuration (`config.js`).
-   `/test_files`: Contains any static files needed for tests, such as sample assignment submissions.

## Prerequisites

1.  **Install k6:** Follow the [official installation guide](https://k6.io/docs/getting-started/installation/).
2.  **Running Application:** Ensure you have a local or staging instance of the application running. The base URL is configured in `shared/config.js`.

## Running Tests

All tests should be run from the root of the `tests/k6` directory. The scripts are configured to use environment variables for dynamic data like usernames, passwords, and IDs.

### Targeting Environments

By default, the tests run against the local environment (`http://localhost:3000/api`). You can target the deployed production environment by setting the `BASE_URL` environment variable.

**WARNING: Running high-load tests against a production environment can impact real users. Do so with caution and during off-peak hours.**

**Example: Running a test against the deployed version:**
```bash
k6 run -e BASE_URL=https://fitchfork.co.za/api -e USERNAME=student -e PASSWORD=1 scenarios/1_login_dashboard_test.js
```

---

### 1. Login and Dashboard Load Test

Simulates users logging in and loading their dashboard.

```bash
k6 run -e USERNAME=student -e PASSWORD=1 scenarios/1_login_dashboard_test.js
```

### 2. Assignment Submission Test

Simulates students submitting an assignment. Requires a sample file in the `/test_files` directory.

```bash
k6 run -e USERNAME=student -e PASSWORD=1 -e MODULE_ID=1 -e ASSIGNMENT_ID=1 scenarios/2_submit_assignment_test.js
```

### 3. View Submissions Test

Simulates a lecturer viewing all submissions for a large assignment.

```bash
k6 run -e LECTURER_USERNAME=lecturer -e LECTURER_PASSWORD=1 -e MODULE_ID=1 -e ASSIGNMENT_ID_LARGE=2 scenarios/3_view_submissions_test.js
```

### 4. Assignment Statistics and Analysis Test

Tests the performance of data-intensive assignment operations including statistics computation, submission analysis, and detailed report generation.

```bash
k6 run -e LECTURER_USERNAME=lecturer -e LECTURER_PASSWORD=1 -e MODULE_ID=1 -e ASSIGNMENT_ID=1 scenarios/4_gradebook_performance_test.js
```

### 5. Create Announcement Test

Simulates a lecturer creating and publishing announcements.

```bash
k6 run -e LECTURER_USERNAME=lecturer -e LECTURER_PASSWORD=1 -e MODULE_ID=1 scenarios/5_create_announcement_test.js
```