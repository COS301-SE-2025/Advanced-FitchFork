-- Create assignments table
CREATE TABLE assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    due_date TEXT,
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE
);