
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_number TEXT NOT NULL UNIQUE CHECK(student_number GLOB 'u[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]'),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    admin BOOLEAN NOT NULL
);