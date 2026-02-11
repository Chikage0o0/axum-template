ALTER TABLE users
ADD COLUMN role TEXT NOT NULL DEFAULT 'user';

UPDATE users
SET role = 'admin'
WHERE username = 'admin';

ALTER TABLE users
ADD CONSTRAINT users_role_check CHECK (role IN ('admin', 'user'));
