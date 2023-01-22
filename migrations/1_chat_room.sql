CREATE TABLE IF NOT EXISTS chat_room (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    owner_id INT UNSIGNED NOT NULL,
    time_created TIMESTAMP NOT NULL,
    last_updated TIMESTAMP NOT NULL
);