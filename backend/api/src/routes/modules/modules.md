# Modules Routes Documentation

**Base Path:** `/modules`

All routes in this group require admin privileges.

---

## POST `/modules`

**Description:**
Create a new university module. A module represents a course such as "COS301" and includes key metadata like its code, academic year, description, and credit value.

**Request Body:**
```json
{
  "code": "COS301",
  "year": 2025,
  "description": "Advanced Software Engineering",
  "credits": 16
}
```

**Validation Rules:**
- `code`: Required, must be uppercase alphanumeric (e.g., `^[A-Z]{3}\d{3}$`), unique
- `year`: Required, must be the current year or later
- `description`: Optional, max length 1000 characters
- `credits`: Required, must be a positive integer

**Response:**
```json
{
  "success": true,
  "data": {
    "id": 1,
    "code": "COS301",
    "year": 2025,
    "description": "Advanced Software Engineering",
    "credits": 16,
    "created_at": "2025-05-23T18:00:00Z",
    "updated_at": "2025-05-23T18:00:00Z"
  },
  "message": "Module created successfully"
}
```

**Status Codes:**
- `201 Created`: Module created successfully
- `400 Bad Request`: Validation failure
- `401 Unauthorized`: Missing or invalid JWT
- `403 Forbidden`: Authenticated but not admin user
- `409 Conflict`: Module code already exists
- `500 Internal Server Error`: Database error

---

## Notes

- All routes require admin privileges
- Module codes must follow the format `ABC123` (3 uppercase letters followed by 3 digits)
- Module codes must be unique
- Year must be current year or later
- Credits must be a positive integer
- Description is optional but limited to 1000 characters
- All timestamps are in ISO 8601 format
