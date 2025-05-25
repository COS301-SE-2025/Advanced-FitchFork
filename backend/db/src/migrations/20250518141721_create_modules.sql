--Bit of a weird system here
--The code can be repeated (e.g. COS132 2024 and COS132 2025)
--So the combination of the code and year needs to be unique
CREATE TABLE modules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL,
    year INTEGER NOT NULL,
    description TEXT,
    credits INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now')),
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