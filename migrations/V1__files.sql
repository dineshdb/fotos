PRAGMA case_sensitive_like = false;
PRAGMA auto_vacuum = 1; -- FULL
PRAGMA journal_mode = wal2; -- different implementation of the atomicity properties
PRAGMA journal_size_limit = 6144000;
PRAGMA foreign_keys = on; -- check foreign key reference, slightly worst performance
PRAGMA analysis_limit=400; -- make sure pragma optimize does not take too long
PRAGMA optimize; -- gather statistics to improve query optimization

PRAGMA busy_timeout = 5000;
PRAGMA cache_size = 1000000000;
PRAGMA temp_store = memory;


CREATE TABLE IF NOT EXISTS files (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  path TEXT UNIQUE NOT NULL,
  size INT NOT NULL,
  type TEXT,

  b3sum TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  modified_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  seen_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
