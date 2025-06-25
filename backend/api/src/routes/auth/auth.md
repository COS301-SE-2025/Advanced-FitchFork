# Authentication Routes Documentation

**Base Path:** `/auth`

---

## POST `/auth/register`

**Description:**
Register a new user account.

**Request Body:**

```json
{
	"username": "u12345678",
	"email": "user@example.com",
	"password": "strongpassword"
}
```

**Validation Rules:**
- Student number must be in format `u12345678`
- Email must be a valid email format
- Password must be at least 8 characters

**Response:**

```json
{
	"success": true,
	"data": {
		"id": 1,
		"username": "u12345678",
		"email": "user@example.com",
		"admin": false,
		"token": "jwt_token_here",
		"expires_at": "2025-05-23T11:00:00Z"
	},
	"message": "User registered successfully"
}
```

**Status Codes:**

- `201 Created`: Registration successful
- `400 Bad Request`: Validation failure
- `409 Conflict`: User with email or student number already exists
- `500 Internal Server Error`: Database error

---

## POST `/auth/login`

**Description:**
Authenticate an existing user and issue a JWT token.

**Request Body:**

```json
{
	"username": "u12345678",
	"password": "strongpassword"
}
```

**Response:**

```json
{
	"success": true,
	"data": {
		"id": 1,
		"username": "u12345678",
		"email": "user@example.com",
		"admin": false,
		"token": "jwt_token_here",
		"expires_at": "2025-05-23T12:00:00Z"
	},
	"message": "Login successful"
}
```

**Status Codes:**

- `200 OK`: Login successful
- `401 Unauthorized`: Invalid credentials
- `500 Internal Server Error`: Database error

---

## Authentication Notes

- Both endpoints return a JWT token upon successful authentication
- The token includes user ID and admin status
- Student numbers must follow the format `u12345678`
- Email addresses must be unique
- Student numbers must be unique
- Passwords are validated for minimum length of 8 characters
