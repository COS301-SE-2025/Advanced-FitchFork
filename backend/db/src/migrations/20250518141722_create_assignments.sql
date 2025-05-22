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