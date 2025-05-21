
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_number TEXT NOT NULL UNIQUE CHECK(student_number GLOB 'u[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]'),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    admin BOOLEAN NOT NULL
);

--Bug here -> module code (e.g. COS132) is unique, but you also specify a year
--You should be able to have COS132 2024 and COS132 2025 (The combination of code and year should be unique)
CREATE TABLE modules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    year INTEGER NOT NULL,
    description TEXT
);

CREATE TABLE module_lecturers (
    module_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    PRIMARY KEY (module_id, user_id),
    FOREIGN KEY (module_id) REFERENCES modules(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE module_tutors (
    module_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    PRIMARY KEY (module_id, user_id),
    FOREIGN KEY (module_id) REFERENCES modules(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE module_students (
    module_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    PRIMARY KEY (module_id, user_id),
    FOREIGN KEY (module_id) REFERENCES modules(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

--This will probably have to change
CREATE TABLE assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    due_date TEXT,
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE
);