# Users Routes Documentation

**Base Path:** `/users`

All routes in this group require admin privileges.

---

## GET `/users`

**Description:**
Retrieve a paginated list of users with optional filtering and sorting.

**Query Parameters:**
- `page` (optional): Page number (default: 1, min: 1)
- `per_page` (optional): Items per page (default: 20, min: 1, max: 100)
- `query` (optional): Case-insensitive partial match against email OR username
- `email` (optional): Case-insensitive partial match on email (ignored if query is provided)
- `username` (optional): Case-insensitive partial match on student number (ignored if query is provided)
- `admin` (optional): Filter by admin status (true/false)
- `sort` (optional): Comma-separated sort fields. Use `-` prefix for descending

**Example Queries:**
```http
GET /users?page=2&per_page=10
GET /users?query=u1234
GET /users?email=@example.com
GET /users?username=u1234
GET /users?admin=true
GET /users?sort=email,-created_at
GET /users?page=1&per_page=10&admin=false&query=jacques&sort=-email
```

**Response:**
```json
{
  "success": true,
  "data": {
    "users": [
      {
        "id": "uuid",
        "email": "user@example.com",
        "username": "u12345678",
        "admin": false,
        "created_at": "2025-05-23T18:00:00Z",
        "updated_at": "2025-05-23T18:00:00Z"
      }
    ],
    "page": 1,
    "per_page": 10,
    "total": 135
  },
  "message": "Users retrieved successfully"
}
```

**Status Codes:**
- `200 OK`: Success
- `400 Bad Request`: Invalid query parameters
- `401 Unauthorized`: Missing or invalid JWT
- `403 Forbidden`: Authenticated but not admin user
- `500 Internal Server Error`: Database error

---

## PUT `/users/:id`

**Description:**
Update a user's information.

**Path Parameters:**
- `id`: The ID of the user to update

**Request Body:**
```json
{
  "username": "u87654321",  // optional
  "email": "new@example.com",     // optional
  "admin": true                   // optional
}
```

**Validation Rules:**
- Student number must be in format `u12345678`
- Email must be a valid email format
- At least one field must be provided for update

**Response:**
```json
{
  "success": true,
  "data": {
    "id": 1,
    "username": "u87654321",
    "email": "new@example.com",
    "admin": true,
    "created_at": "2025-05-23T18:00:00Z",
    "updated_at": "2025-05-23T18:00:00Z"
  },
  "message": "User updated successfully"
}
```

**Status Codes:**
- `200 OK`: Update successful
- `400 Bad Request`: Validation error or no fields provided
- `401 Unauthorized`: Missing or invalid JWT
- `403 Forbidden`: Authenticated but not admin user
- `404 Not Found`: User doesn't exist
- `409 Conflict`: Duplicate email or student number
- `500 Internal Server Error`: Database error

---

## DELETE `/users/:id`

**Description:**
Delete a user by their ID. Users cannot delete their own account.

**Path Parameters:**
- `id`: The ID of the user to delete

**Response:**
```json
{
  "success": true,
  "message": "User deleted successfully"
}
```

**Status Codes:**
- `200 OK`: Deletion successful
- `400 Bad Request`: Invalid user ID format
- `401 Unauthorized`: Missing or invalid JWT
- `403 Forbidden`: Attempting to delete own account
- `404 Not Found`: User doesn't exist
- `500 Internal Server Error`: Database error

---

## GET `/users/:id/modules`

**Description:**
Retrieve all modules that a specific user is involved in, including their role in each module.

**Path Parameters:**
- `id`: The ID of the user to fetch modules for

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "code": "COS301",
      "year": 2025,
      "description": "Advanced Software Engineering",
      "credits": 16,
      "role": "Lecturer",
      "created_at": "2025-05-01T08:00:00Z",
      "updated_at": "2025-05-01T08:00:00Z"
    }
  ],
  "message": "Modules for user retrieved successfully"
}
```

**Status Codes:**
- `200 OK`: Success
- `400 Bad Request`: Invalid user ID format
- `401 Unauthorized`: Missing or invalid JWT
- `403 Forbidden`: Authenticated but not admin user
- `404 Not Found`: User doesn't exist
- `500 Internal Server Error`: Database error

---

## Notes

- All routes require admin privileges
- Student numbers must follow the format `u12345678`
- Email addresses must be unique
- Student numbers must be unique
- Users cannot delete their own accounts
- The list users endpoint supports flexible filtering and sorting
- All timestamps are in ISO 8601 format
