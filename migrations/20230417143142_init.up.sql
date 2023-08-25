CREATE TABLE
    IF NOT EXISTS users (
                id VARCHAR(48) PRIMARY KEY NOT NULL,
                name VARCHAR(256) NOT NULL,
                mail VARCHAR(256) NOT NULL
    );