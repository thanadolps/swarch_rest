-- Add migration script here
CREATE TABLE IF NOT EXISTS `tasks` (
  `id` INTEGER PRIMARY KEY NOT NULL,
  `title` varchar(255) NOT NULL,
  `description` text,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `completed` bool NOT NULL DEFAULT false
);
