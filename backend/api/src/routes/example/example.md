# Example Routes Documentation

**Base Path:** `/example`

This route group is used for internal demonstration and testing of Axum routing and middleware, including JWT-based authentication.

---

## GET `/example`

**Description:**
Public route that returns a simple confirmation message.

**Response:**

```json
{
	"success": true,
	"data": "Example index",
	"message": "Fetched list"
}
```

**Status Codes:**

- `200 OK`: Request succeeded

---

## POST `/example`

**Description:**
Creates a new example resource (dummy route for demonstration). No authentication required.

**Request Body:**

```json
{
	"name": "string",
	"value": "any"
}
```

**Response:**

```json
{
	"success": true,
	"data": "Created",
	"message": "Resource created"
}
```

**Status Codes:**

- `201 Created`: Resource successfully created
- `400 Bad Request`: Invalid input

---

## DELETE `/example/:id`

**Description:**
Deletes an example resource by ID. This route is **protected** (authentication middleware to be finalized).

**Path Parameter:**

- `id` (integer): The ID of the resource to delete.

**Response:**

```json
{
	"success": true,
	"data": "Deleted item 42",
	"message": "Resource deleted"
}
```

**Status Codes:**

- `200 OK`: Deletion successful
- `401 Unauthorized`: If authentication fails

---

## GET `/example/auth`

**Description:**
Authenticated route. Requires a valid JWT. Returns the user ID extracted from token claims.

**Response:**

```json
{
	"success": true,
	"data": "Test Get Route Auth",
	"message": "Well done you authenticated, good boy user with id: 123"
}
```

**Status Codes:**

- `200 OK`: Success
- `401 Unauthorized`: Missing or invalid token

---

## GET `/example/admin`

**Description:**
Admin-only route. Requires a valid JWT with `admin = true` claim.

**Response:**

```json
{
	"success": true,
	"data": "Test Get Route Admin",
	"message": "Well done you are an Admin, good boy. Your user ID is: 123"
}
```

**Status Codes:**

- `200 OK`: Success
- `401 Unauthorized`: Invalid/missing token
- `403 Forbidden`: Authenticated but not an admin

---

## GET `/example/admin-auth`

**Description:**
Demonstrates stacked middleware: both `require_authenticated` and `require_admin` are applied.

**Response:**

```json
{
	"success": true,
	"data": "Test Get Route Admin + Auth",
	"message": "You passed both auth layers. Welcome, user ID 123"
}
```

**Status Codes:**

- `200 OK`: Success
- `401 Unauthorized`: Invalid/missing token
- `403 Forbidden`: Not an admin

---

## Authentication

- `GET /example`, `POST /example` → Public routes
- `GET /example/auth` → Requires `require_authenticated`
- `GET /example/admin` → Requires `require_admin`
- `GET /example/admin-auth` → Requires both `require_authenticated` and `require_admin`
- `DELETE /example/:id` → Middleware pending

Ensure the JWT token is provided in the `Authorization: Bearer <token>` header for protected routes.