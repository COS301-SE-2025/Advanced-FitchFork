
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_number TEXT NOT NULL UNIQUE CHECK(student_number GLOB 'u[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]'),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    admin BOOLEAN NOT NULL
);

--Bit of a weird system here
--The code can be repeated (e.g. COS132 2024 and COS132 2025)
--So the combination of the code and year needs to be unique
CREATE TABLE modules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL,
    year INTEGER NOT NULL,
    description TEXT,
    UNIQUE (code, year)
);

--Relationships between tables
--On Delete Cascade very important
CREATE TABLE module_lecturers (
    module_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    PRIMARY KEY (module_id, user_id),
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE module_tutors (
    module_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    PRIMARY KEY (module_id, user_id),
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE module_students (
    module_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    PRIMARY KEY (module_id, user_id),
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- --This will probably have to change
-- CREATE TABLE assignments (
--     id INTEGER PRIMARY KEY AUTOINCREMENT,
--     module_id INTEGER NOT NULL,
--     name TEXT NOT NULL,
--     due_date TEXT,
--     FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE
-- );

CREATE TABLE assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    assignment_type TEXT NOT NULL CHECK (assignment_type IN ('Assignment', 'Practical')),
    available_from DATETIME NOT NULL,
    due_date DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE
);

--But why :(
CREATE TRIGGER update_assignment_updated_at
BEFORE UPDATE ON assignments
FOR EACH ROW
BEGIN
    UPDATE assignments
    SET updated_at = datetime('now')
    WHERE id = OLD.id;
END;

CREATE TABLE assignment_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    assignment_id INTEGER NOT NULL,
    filename TEXT NOT NULL,
    path TEXT NOT NULL,
    uploaded_at DATETIME NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (assignment_id) REFERENCES assignments(id) ON DELETE CASCADE,
    UNIQUE (assignment_id, filename)
);