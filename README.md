# `rustyrc`

(School project) Basic IRC server implemented in Rust.

## Requirements
- MySQL/MariaDB,
- `cargo`
- `git`

## Setup
- Create a database for it,
- Edit `.env` with corresponding database URL
- Create tables using:
```sql
CREATE TABLE `bans` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `is_ip` tinyint(1) NOT NULL,
  `content` char(20) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `channels` (
                            `id` int(11) NOT NULL AUTO_INCREMENT,
                            `name` char(15) NOT NULL DEFAULT '',
                            `creation_time` int(12) NOT NULL,
                            `creator` char(11) NOT NULL DEFAULT '',
                            `motd` mediumtext NOT NULL,
                            `content` longtext NOT NULL,
                            PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `settings` (
                            `id` int(11) NOT NULL AUTO_INCREMENT,
                            `key` char(11) NOT NULL DEFAULT '',
                            `content` text NOT NULL,
                            PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `users` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `last_login` bigint(11) NOT NULL,
  `nick` char(11) NOT NULL DEFAULT '',
  `real_name` char(25) NOT NULL DEFAULT '',
  `last_ip` char(11) NOT NULL DEFAULT '',
  `is_connected` tinyint(1) NOT NULL,
  `op` tinyint(1) NOT NULL,
  `thread_id` int(11) NOT NULL DEFAULT 0,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=30 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
```
- Run!