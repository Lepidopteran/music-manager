CREATE TABLE `directories` (
    `name` TEXT NOT NULL PRIMARY KEY,
    `path` TEXT NOT NULL,
    UNIQUE (`path`) ON CONFLICT REPLACE
);
