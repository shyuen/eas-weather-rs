CREATE TABLE IF NOT EXISTS `Alerts` (
  `Identifier` varchar(128) NOT NULL,
  `Sender` longtext DEFAULT NULL,
  `Sent` datetime(6) NOT NULL,
  `Status` longtext DEFAULT NULL,
  `MsgType` longtext DEFAULT NULL,
  `Source` longtext DEFAULT NULL,
  `Scope` longtext DEFAULT NULL,
  `References` longtext DEFAULT NULL,
  PRIMARY KEY (`Identifier`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
