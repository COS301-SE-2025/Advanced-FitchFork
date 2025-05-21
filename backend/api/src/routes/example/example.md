# Example Routes Documentation

**Base Path:** `/example`

---

## GET `/example`

**Description:**
Returns a simple confirmation message for the example route.

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
Creates a new example resource (dummy route for demonstration).

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
Deletes an example resource by ID. This route is protected and requires authentication.

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

## Authentication

- `GET /example` and `POST /example` do **not** require authentication.
- `DELETE /example/:id` **requires authentication** via middleware (`dummy_auth`).

Update this section if additional access control or JWT validation is added in the future.
