# Users Routes Documentation

**Base Path:** `/users`

---

## GET `/users`

**Description:**
Retrieve a list of all users in the system.

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "student_number": "u12345678",
      "email": "user@example.com",
      "name": "John Doe",
      "role": "User",
      "admin": false
    }
  ],
  "message": "Users retrieved successfully"
}
```

**Status Codes:**
- `200 OK`: Successfully retrieved users
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not authorized to view users
- `500 Internal Server Error`: Database error

---

## GET `/users/:id`

**Description:**
Retrieve details of a specific user by ID.

**Response:**

```json
{
  "success": true,
  "data": {
    "id": 1,
    "student_number": "u12345678",
    "email": "user@example.com",
    "name": "John Doe",
    "role": "User",
    "admin": false
  },
  "message": "User retrieved successfully"
}
```

**Status Codes:**
- `200 OK`: Successfully retrieved user
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not authorized to view user
- `404 Not Found`: User not found
- `500 Internal Server Error`: Database error

---

## PUT `/users/:id`

**Description:**
Update a user's information.

**Request Body:**

```json
{
  "name": "John Doe",
  "email": "john.doe@example.com",
  "role": "Moderator"
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "id": 1,
    "student_number": "u12345678",
    "email": "john.doe@example.com",
    "name": "John Doe",
    "role": "Moderator",
    "admin": false
  },
  "message": "User updated successfully"
}
```

**Status Codes:**
- `200 OK`: Successfully updated user
- `400 Bad Request`: Invalid input data
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not authorized to update user
- `404 Not Found`: User not found
- `409 Conflict`: Email already in use
- `500 Internal Server Error`: Database error

---

## DELETE `/users/:id`

**Description:**
Delete a user from the system.

**Response:**

```json
{
  "success": true,
  "message": "User deleted successfully"
}
```

**Status Codes:**
- `200 OK`: Successfully deleted user
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not authorized to delete user
- `404 Not Found`: User not found
- `500 Internal Server Error`: Database error

---

## User Management Notes

- All endpoints require authentication via JWT token
- User roles include: Admin, Moderator, and User
- Only admins can modify user roles
- Users can only modify their own information (except admins)
- Student numbers cannot be modified after creation
- Email addresses must be unique across all users
- User deletion is permanent and cannot be undone
- The system maintains an audit log of all user modifications
