CREATE TABLE IF NOT EXISTS chat_users (
    chat_room_id INT UNSIGNED NOT NULL,
    user_id INT UNSIGNED NOT NULL,
    time_joined TIMESTAMP NOT NULL
);