CREATE TABLE
    todos (
        id UUID PRIMARY KEY,
        title VARCHAR(100) NOT NULL,
        description VARCHAR(1000),
        is_done BOOLEAN NOT NULL DEFAULT FALSE,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );