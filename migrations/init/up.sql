-- Your SQL goes here

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
                            `topic` mediumtext NOT NULL,
                            `content` longtext NOT NULL,
                            PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=1 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
CREATE TABLE `settings` (
                            `id` int(11) NOT NULL AUTO_INCREMENT,
                            `key` char(11) NOT NULL DEFAULT '',
                            `content` text NOT NULL,
                            PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=1 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

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
) ENGINE=InnoDB AUTO_INCREMENT=1 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `memberships` (
                               `id` int(11) NOT NULL AUTO_INCREMENT,
                               `id_user` int(11) NOT NULL,
                               `id_channel` int(11) NOT NULL,
                               PRIMARY KEY (`id`),
                               KEY `user` (`id_user`),
                               KEY `channel` (`id_channel`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

INSERT INTO `settings` (`id`, `key`, `content`)
VALUES
    (1, 'ip', '127.0.0.1'),
    (2, 'port', '6667'),
    (3, 'name', 'CompanyChat'),
    (4, 'motd', 'Bienvenue chez Company');

INSERT INTO `users` (`id`, `last_login`, `nick`, `real_name`, `last_ip`, `is_connected`, `op`, `thread_id`)
VALUES
    (1, 0, 'system', 'system', '127.0.0.1', 0, 1, -1);

INSERT INTO `channels` (`id`, `name`, `creation_time`, `creator`, `topic`, `content`)
VALUES
    (2, '#general', 11, 'system', 'Salon général', ' ');
