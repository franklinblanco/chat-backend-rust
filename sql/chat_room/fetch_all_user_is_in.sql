SELECT cr.* FROM chat_room cr
LEFT JOIN chat_users cu ON cu.chat_room_id = cr.id
WHERE cu.user_id = ?
ORDER BY cu.time_joined DESC