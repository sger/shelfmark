CREATE TABLE book_user_state (
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
  favorite BOOLEAN NOT NULL DEFAULT false,
  reading_status TEXT NOT NULL DEFAULT 'unread' CHECK (reading_status IN ('unread', 'reading', 'finished')),
  status_overridden BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, book_id)
);

CREATE TABLE tags (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_tags_user_lower_name ON tags(user_id, lower(name));

CREATE TABLE book_tags (
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
  tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, book_id, tag_id)
);

CREATE TABLE smart_shelves (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  match_mode TEXT NOT NULL DEFAULT 'all' CHECK (match_mode IN ('all', 'any')),
  rules JSONB NOT NULL DEFAULT '[]'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_book_user_state_user_status ON book_user_state(user_id, reading_status);
CREATE INDEX idx_book_user_state_user_favorite ON book_user_state(user_id, favorite);
CREATE INDEX idx_book_tags_user_book ON book_tags(user_id, book_id);
CREATE INDEX idx_smart_shelves_user ON smart_shelves(user_id);
