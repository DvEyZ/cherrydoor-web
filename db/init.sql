CREATE TABLE `users` (
  `id` int PRIMARY KEY AUTO_INCREMENT,
  `name` varchar(255) UNIQUE NOT NULL,
  `full_name` varchar(255) NOT NULL,
  `role` varchar(255) NOT NULL
);

CREATE TABLE `access_codes` (
  `id` int PRIMARY KEY AUTO_INCREMENT,
  `code` varchar(255) UNIQUE NOT NULL,
  `user` int NOT NULL
);

CREATE TABLE `access_profiles` (
  `id` int PRIMARY KEY AUTO_INCREMENT,
  `name` varchar(255) UNIQUE NOT NULL,
  `description` varchar(255) NOT NULL,
  `display_text` varchar(255) NOT NULL,
  `color` varchar(255) NOT NULL,
  `access_mode` ENUM ('OpenLock', 'AllowAnyone', 'CheckAccess') NOT NULL
);

CREATE TABLE `permissions` (
  `id` int PRIMARY KEY AUTO_INCREMENT,
  `name` varchar(255) UNIQUE NOT NULL,
  `description` varchar(255) NOT NULL
);

CREATE TABLE `access_profiles_permissions` (
  `access_profile_id` int NOT NULL,
  `permission_id` int NOT NULL,
  PRIMARY KEY (`access_profile_id`, `permission_id`)
);

CREATE TABLE `users_permissions` (
  `user_id` int NOT NULL,
  `permission_id` int NOT NULL,
  PRIMARY KEY (`user_id`, `permission_id`)
);

CREATE TABLE `web_ui_users` (
  `id` int PRIMARY KEY AUTO_INCREMENT,
  `name` varchar(255) UNIQUE NOT NULL,
  `password_hash` varchar(255) NOT NULL,
  `is_admin` boolean NOT NULL,
  `ac_does_not_expire` boolean NOT NULL
);

ALTER TABLE `access_codes` ADD FOREIGN KEY (`user`) REFERENCES `users` (`id`);

ALTER TABLE `access_profiles_permissions` ADD FOREIGN KEY (`access_profile_id`) REFERENCES `access_profiles` (`id`);

ALTER TABLE `access_profiles_permissions` ADD FOREIGN KEY (`permission_id`) REFERENCES `permissions` (`id`);

ALTER TABLE `users_permissions` ADD FOREIGN KEY (`user_id`) REFERENCES `users` (`id`);

ALTER TABLE `users_permissions` ADD FOREIGN KEY (`permission_id`) REFERENCES `permissions` (`id`);

