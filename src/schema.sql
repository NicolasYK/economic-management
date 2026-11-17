PRAGMA application_id = 1162692401;
PRAGMA user_version = 1;
CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY CHECK(version=1)) STRICT;
INSERT INTO schema_migrations VALUES(1);
CREATE TABLE settings(
 id INTEGER PRIMARY KEY CHECK(id=1), opening INTEGER NOT NULL CHECK(opening BETWEEN -1000000000000 AND 1000000000000),
 reserve INTEGER NOT NULL DEFAULT 0 CHECK(reserve BETWEEN 0 AND 1000000000000),
 theme TEXT NOT NULL DEFAULT 'system' CHECK(theme IN ('light','dark','system')),
 start INTEGER NOT NULL CHECK(start BETWEEN 24000 AND 25211),
 last_seen INTEGER NOT NULL CHECK(last_seen BETWEEN start AND 25211)
) STRICT;
CREATE TABLE income_schedules(month INTEGER PRIMARY KEY CHECK(month BETWEEN 24000 AND 25211),amount INTEGER NOT NULL CHECK(amount BETWEEN 0 AND 1000000000000)) STRICT;
CREATE TABLE categories(id INTEGER PRIMARY KEY,name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 80),normalized TEXT NOT NULL UNIQUE) STRICT;
INSERT INTO categories VALUES(1,'Alimentação','alimentação'),(2,'Hobby','hobby');
CREATE TABLE commitments(
 id INTEGER PRIMARY KEY,description TEXT NOT NULL CHECK(length(description) BETWEEN 1 AND 160),
 category_id INTEGER NOT NULL REFERENCES categories(id),amount INTEGER NOT NULL CHECK(amount BETWEEN 1 AND 1000000000000),
 installments INTEGER CHECK(installments BETWEEN 1 AND 360 AND amount>=installments),
 start INTEGER NOT NULL CHECK(start BETWEEN 24000 AND 25211),
 end INTEGER CHECK(end BETWEEN start AND 25211),cancelled INTEGER CHECK(cancelled BETWEEN start AND 25211)
) STRICT;
CREATE TABLE commitment_revisions(
 commitment_id INTEGER NOT NULL REFERENCES commitments(id),month INTEGER NOT NULL CHECK(month BETWEEN 24000 AND 25211),
 amount INTEGER NOT NULL CHECK(amount BETWEEN 1 AND 1000000000000),PRIMARY KEY(commitment_id,month)
) STRICT;
CREATE TABLE transactions(
 id INTEGER PRIMARY KEY,kind TEXT NOT NULL CHECK(kind IN ('income','expense')),
 description TEXT NOT NULL CHECK(length(description) BETWEEN 1 AND 160),
 amount INTEGER NOT NULL CHECK(amount BETWEEN 0 AND 1000000000000 AND (kind='income' OR amount>0)),
 category_id INTEGER REFERENCES categories(id),date TEXT NOT NULL CHECK(length(date)=10),
 occurrence TEXT UNIQUE,commitment_id INTEGER REFERENCES commitments(id),
 CHECK((kind='income' AND category_id IS NULL AND commitment_id IS NULL AND occurrence IS NOT NULL)
 OR (kind='expense' AND category_id IS NOT NULL AND ((occurrence IS NULL AND commitment_id IS NULL) OR (occurrence IS NOT NULL AND commitment_id IS NOT NULL))))
) STRICT;
CREATE INDEX transactions_date ON transactions(date);
