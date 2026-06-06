use sqlx::{Postgres, QueryBuilder, PgPool};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{BookListItem, BookUserState, CreateShelfRequest, ShelfRule, ShelfSummary, SmartShelf, UpdateBookStateRequest, UpdateShelfRequest},
};

#[derive(Debug, Clone, Default)]
pub struct BookFilters {
    pub q: Option<String>,
    pub library_id: Option<Uuid>,
    pub status: Option<String>,
    pub favorite: Option<bool>,
    pub tag: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub year: Option<String>,
    pub shelf_id: Option<String>,
}

const BUILTIN_SHELVES: [(&str, &str, &str); 6] = [
    ("unread", "Unread", "status"),
    ("in-progress", "In Progress", "status"),
    ("finished", "Finished", "status"),
    ("favorites", "Favorites", "state"),
    ("recently-added", "Recently Added", "recent"),
    ("recently-updated", "Recently Updated", "recent"),
];

pub fn validate_status(status: &str) -> AppResult<()> {
    match status {
        "unread" | "reading" | "finished" => Ok(()),
        _ => Err(AppError::BadRequest("reading_status must be unread, reading, or finished".to_string())),
    }
}

pub async fn list_books(db: &PgPool, user_id: Uuid, filters: BookFilters) -> AppResult<Vec<BookListItem>> {
    if let Some(status) = filters.status.as_deref() {
        validate_status(status)?;
    }

    let mut builder = base_book_query(user_id);
    apply_filters(db, &mut builder, &filters, user_id).await?;
    builder.push(" GROUP BY b.id, s.favorite, s.reading_status, rp.progress_percent");
    if matches!(filters.shelf_id.as_deref(), Some("recently-added")) {
        builder.push(" ORDER BY b.created_at DESC");
    } else {
        builder.push(" ORDER BY b.updated_at DESC");
    }

    Ok(builder.build_query_as::<BookListItem>().fetch_all(db).await?)
}

pub async fn get_book_state(db: &PgPool, user_id: Uuid, book_id: Uuid) -> AppResult<BookUserState> {
    ensure_state(db, user_id, book_id).await?;
    Ok(sqlx::query_as::<_, BookUserState>(
        "SELECT s.user_id, s.book_id, s.favorite, s.reading_status,
                COALESCE(array_agg(t.name ORDER BY lower(t.name)) FILTER (WHERE t.name IS NOT NULL), '{}') AS tags,
                s.updated_at
         FROM book_user_state s
         LEFT JOIN book_tags bt ON bt.user_id = s.user_id AND bt.book_id = s.book_id
         LEFT JOIN tags t ON t.id = bt.tag_id
         WHERE s.user_id = $1 AND s.book_id = $2
         GROUP BY s.user_id, s.book_id, s.favorite, s.reading_status, s.updated_at",
    )
    .bind(user_id)
    .bind(book_id)
    .fetch_one(db)
    .await?)
}

pub async fn update_book_state(db: &PgPool, user_id: Uuid, book_id: Uuid, input: UpdateBookStateRequest) -> AppResult<BookUserState> {
    let current = get_book_state(db, user_id, book_id).await?;
    let favorite = input.favorite.unwrap_or(current.favorite);
    let reading_status = input.reading_status.unwrap_or(current.reading_status);
    validate_status(&reading_status)?;

    let mut tx = db.begin().await?;
    sqlx::query(
        "INSERT INTO book_user_state (user_id, book_id, favorite, reading_status, status_overridden)
         VALUES ($1, $2, $3, $4, true)
         ON CONFLICT (user_id, book_id)
         DO UPDATE SET favorite = excluded.favorite, reading_status = excluded.reading_status,
                       status_overridden = true, updated_at = now()",
    )
    .bind(user_id)
    .bind(book_id)
    .bind(favorite)
    .bind(&reading_status)
    .execute(&mut *tx)
    .await?;

    if let Some(tags) = input.tags {
        sqlx::query("DELETE FROM book_tags WHERE user_id = $1 AND book_id = $2")
            .bind(user_id)
            .bind(book_id)
            .execute(&mut *tx)
            .await?;

        for tag in normalize_tags(tags) {
            let tag_id: Uuid = sqlx::query_scalar(
                "INSERT INTO tags (user_id, name) VALUES ($1, $2)
                 ON CONFLICT (user_id, lower(name)) DO UPDATE SET name = excluded.name
                 RETURNING id",
            )
            .bind(user_id)
            .bind(&tag)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query(
                "INSERT INTO book_tags (user_id, book_id, tag_id) VALUES ($1, $2, $3)
                 ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .bind(book_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    get_book_state(db, user_id, book_id).await
}

pub async fn derive_reading_status(db: &PgPool, user_id: Uuid, book_id: Uuid, progress_percent: f64) -> AppResult<()> {
    let derived = if progress_percent > 0.0 { "reading" } else { "unread" };
    sqlx::query(
        "INSERT INTO book_user_state (user_id, book_id, reading_status, status_overridden)
         VALUES ($1, $2, $3, false)
         ON CONFLICT (user_id, book_id)
         DO UPDATE SET reading_status = CASE
                 WHEN book_user_state.status_overridden THEN book_user_state.reading_status
                 ELSE excluded.reading_status
               END,
               updated_at = CASE
                 WHEN book_user_state.status_overridden THEN book_user_state.updated_at
                 ELSE now()
               END",
    )
    .bind(user_id)
    .bind(book_id)
    .bind(derived)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn list_shelves(db: &PgPool, user_id: Uuid) -> AppResult<Vec<ShelfSummary>> {
    let mut shelves = Vec::new();
    for (id, name, group) in BUILTIN_SHELVES {
        let count = list_books(db, user_id, BookFilters { shelf_id: Some(id.to_string()), ..Default::default() }).await?.len() as i64;
        shelves.push(ShelfSummary { id: id.to_string(), name: name.to_string(), kind: "built-in".to_string(), count, group: Some(group.to_string()), rules: None, match_mode: None });
    }

    for (prefix, group, label) in [
        ("author", "author", "Author"),
        ("language", "language", "Language"),
        ("year", "year", "Year"),
        ("tag", "tag", "Tag"),
    ] {
        for (value, count) in group_counts(db, user_id, prefix).await? {
            shelves.push(ShelfSummary {
                id: format!("{prefix}:{value}"),
                name: format!("{label}: {value}"),
                kind: "built-in".to_string(),
                count,
                group: Some(group.to_string()),
                rules: None,
                match_mode: None,
            });
        }
    }

    let custom = sqlx::query_as::<_, SmartShelf>(
        "SELECT id, user_id, name, match_mode, rules, created_at, updated_at
         FROM smart_shelves WHERE user_id = $1 ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    for shelf in custom {
        let rules = parse_rules(&shelf.rules)?;
        let count = list_books(db, user_id, BookFilters { shelf_id: Some(shelf.id.to_string()), ..Default::default() }).await?.len() as i64;
        shelves.push(ShelfSummary {
            id: shelf.id.to_string(),
            name: shelf.name,
            kind: "custom".to_string(),
            count,
            group: Some("custom".to_string()),
            rules: Some(rules),
            match_mode: Some(shelf.match_mode),
        });
    }

    Ok(shelves)
}

pub async fn create_shelf(db: &PgPool, user_id: Uuid, input: CreateShelfRequest) -> AppResult<ShelfSummary> {
    validate_shelf_input(&input.name, &input.match_mode, &input.rules)?;
    let rules = serde_json::to_value(&input.rules).map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?;
    let shelf = sqlx::query_as::<_, SmartShelf>(
        "INSERT INTO smart_shelves (user_id, name, match_mode, rules)
         VALUES ($1, $2, $3, $4)
         RETURNING id, user_id, name, match_mode, rules, created_at, updated_at",
    )
    .bind(user_id)
    .bind(input.name.trim())
    .bind(input.match_mode)
    .bind(rules)
    .fetch_one(db)
    .await?;
    shelf_summary(db, user_id, shelf).await
}

pub async fn update_shelf(db: &PgPool, user_id: Uuid, id: Uuid, input: UpdateShelfRequest) -> AppResult<ShelfSummary> {
    let existing = find_smart_shelf(db, user_id, id).await?;
    let rules = if let Some(rules) = input.rules { rules } else { parse_rules(&existing.rules)? };
    let name = input.name.unwrap_or(existing.name);
    let match_mode = input.match_mode.unwrap_or(existing.match_mode);
    validate_shelf_input(&name, &match_mode, &rules)?;
    let rules_json = serde_json::to_value(&rules).map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?;
    let shelf = sqlx::query_as::<_, SmartShelf>(
        "UPDATE smart_shelves SET name = $1, match_mode = $2, rules = $3, updated_at = now()
         WHERE id = $4 AND user_id = $5
         RETURNING id, user_id, name, match_mode, rules, created_at, updated_at",
    )
    .bind(name.trim())
    .bind(match_mode)
    .bind(rules_json)
    .bind(id)
    .bind(user_id)
    .fetch_one(db)
    .await?;
    shelf_summary(db, user_id, shelf).await
}

pub async fn delete_shelf(db: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM smart_shelves WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

async fn ensure_state(db: &PgPool, user_id: Uuid, book_id: Uuid) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO book_user_state (user_id, book_id) VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(book_id)
    .execute(db)
    .await?;
    Ok(())
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for tag in tags {
        let trimmed = tag.trim();
        if !trimmed.is_empty() && !out.iter().any(|existing: &String| existing.eq_ignore_ascii_case(trimmed)) {
            out.push(trimmed.to_string());
        }
    }
    out.truncate(20);
    out
}

fn base_book_query(user_id: Uuid) -> QueryBuilder<'static, Postgres> {
    let mut builder = QueryBuilder::new(
        "SELECT b.id, b.library_id, b.title, b.authors, b.description, b.publisher, b.published_date, b.language,
                b.page_count, b.cover_path, b.created_at, b.updated_at,
                COALESCE(s.favorite, false) AS favorite,
                COALESCE(s.reading_status, 'unread') AS reading_status,
                COALESCE(array_agg(DISTINCT t.name ORDER BY t.name) FILTER (WHERE t.name IS NOT NULL), '{}') AS tags,
                rp.progress_percent AS progress_percent
         FROM books b
         LEFT JOIN book_user_state s ON s.book_id = b.id AND s.user_id = ",
    );
    builder.push_bind(user_id);
    builder.push(
        " LEFT JOIN reading_progress rp ON rp.book_id = b.id AND rp.user_id = ",
    );
    builder.push_bind(user_id);
    builder.push(
        " LEFT JOIN book_tags bt ON bt.book_id = b.id AND bt.user_id = ",
    );
    builder.push_bind(user_id);
    builder.push(" LEFT JOIN tags t ON t.id = bt.tag_id WHERE 1 = 1");
    builder
}

async fn apply_filters(db: &PgPool, builder: &mut QueryBuilder<'static, Postgres>, filters: &BookFilters, user_id: Uuid) -> AppResult<()> {
    if let Some(q) = filters.q.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        let like = format!("%{q}%");
        builder.push(" AND (b.title ILIKE ").push_bind(like.clone()).push(" OR array_to_string(b.authors, ' ') ILIKE ").push_bind(like).push(")");
    }
    if let Some(library_id) = filters.library_id {
        builder.push(" AND b.library_id = ").push_bind(library_id);
    }
    if let Some(status) = filters.status.as_deref() {
        builder.push(" AND COALESCE(s.reading_status, 'unread') = ").push_bind(status.to_string());
    }
    if let Some(favorite) = filters.favorite {
        builder.push(" AND COALESCE(s.favorite, false) = ").push_bind(favorite);
    }
    if let Some(tag) = filters.tag.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        builder.push(" AND EXISTS (SELECT 1 FROM book_tags xbt JOIN tags xt ON xt.id = xbt.tag_id WHERE xbt.user_id = ")
            .push_bind(user_id)
            .push(" AND xbt.book_id = b.id AND lower(xt.name) = lower(")
            .push_bind(tag.to_string())
            .push("))");
    }
    if let Some(author) = filters.author.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        builder.push(" AND EXISTS (SELECT 1 FROM unnest(b.authors) a WHERE a ILIKE ").push_bind(format!("%{author}%")).push(")");
    }
    if let Some(language) = filters.language.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        builder.push(" AND lower(COALESCE(b.language, '')) = lower(").push_bind(language.to_string()).push(")");
    }
    if let Some(year) = filters.year.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        builder.push(" AND left(COALESCE(b.published_date, ''), 4) = ").push_bind(year.to_string());
    }
    if let Some(shelf_id) = filters.shelf_id.as_deref() {
        apply_shelf_filter(db, builder, user_id, shelf_id).await?;
    }
    Ok(())
}

async fn apply_shelf_filter(db: &PgPool, builder: &mut QueryBuilder<'static, Postgres>, user_id: Uuid, shelf_id: &str) -> AppResult<()> {
    match shelf_id {
        "unread" => { builder.push(" AND COALESCE(s.reading_status, 'unread') = 'unread'"); }
        "in-progress" => { builder.push(" AND COALESCE(s.reading_status, 'unread') = 'reading'"); }
        "finished" => { builder.push(" AND COALESCE(s.reading_status, 'unread') = 'finished'"); }
        "favorites" => { builder.push(" AND COALESCE(s.favorite, false) = true"); }
        "recently-added" => { builder.push(" AND b.created_at >= now() - interval '30 days'"); }
        "recently-updated" => { builder.push(" AND b.updated_at >= now() - interval '30 days'"); }
        _ => {
            if let Some(value) = shelf_id.strip_prefix("author:") {
                builder.push(" AND EXISTS (SELECT 1 FROM unnest(b.authors) a WHERE lower(a) = lower(").push_bind(value.to_string()).push("))");
            } else if let Some(value) = shelf_id.strip_prefix("language:") {
                builder.push(" AND lower(COALESCE(b.language, '')) = lower(").push_bind(value.to_string()).push(")");
            } else if let Some(value) = shelf_id.strip_prefix("year:") {
                builder.push(" AND left(COALESCE(b.published_date, ''), 4) = ").push_bind(value.to_string());
            } else if let Some(value) = shelf_id.strip_prefix("tag:") {
                builder.push(" AND EXISTS (SELECT 1 FROM book_tags xbt JOIN tags xt ON xt.id = xbt.tag_id WHERE xbt.user_id = ")
                    .push_bind(user_id)
                    .push(" AND xbt.book_id = b.id AND lower(xt.name) = lower(")
                    .push_bind(value.to_string())
                    .push("))");
            } else {
                let id = shelf_id.parse::<Uuid>().map_err(|_| AppError::NotFound)?;
                let shelf = find_smart_shelf(db, user_id, id).await?;
                apply_custom_rules(builder, user_id, &shelf.match_mode, &parse_rules(&shelf.rules)?)?;
            }
        }
    };
    Ok(())
}

fn apply_custom_rules(builder: &mut QueryBuilder<'static, Postgres>, user_id: Uuid, match_mode: &str, rules: &[ShelfRule]) -> AppResult<()> {
    if rules.is_empty() {
        return Ok(());
    }
    let joiner = if match_mode == "any" { " OR " } else { " AND " };
    builder.push(" AND (");
    for (index, rule) in rules.iter().enumerate() {
        if index > 0 {
            builder.push(joiner);
        }
        apply_rule(builder, user_id, rule)?;
    }
    builder.push(")");
    Ok(())
}

fn apply_rule(builder: &mut QueryBuilder<'static, Postgres>, user_id: Uuid, rule: &ShelfRule) -> AppResult<()> {
    let text = rule.value.as_str().unwrap_or_default().trim().to_string();
    match (rule.field.as_str(), rule.operator.as_str()) {
        ("title", "contains") => builder.push("b.title ILIKE ").push_bind(format!("%{text}%")),
        ("title", "equals") => builder.push("lower(b.title) = lower(").push_bind(text).push(")"),
        ("author", "contains") => builder.push("EXISTS (SELECT 1 FROM unnest(b.authors) a WHERE a ILIKE ").push_bind(format!("%{text}%")).push(")"),
        ("author", "equals") => builder.push("EXISTS (SELECT 1 FROM unnest(b.authors) a WHERE lower(a) = lower(").push_bind(text).push("))"),
        ("language", "equals") => builder.push("lower(COALESCE(b.language, '')) = lower(").push_bind(text).push(")"),
        ("year", "equals") => builder.push("left(COALESCE(b.published_date, ''), 4) = ").push_bind(text),
        ("tag", "equals") => builder.push("EXISTS (SELECT 1 FROM book_tags xbt JOIN tags xt ON xt.id = xbt.tag_id WHERE xbt.user_id = ").push_bind(user_id).push(" AND xbt.book_id = b.id AND lower(xt.name) = lower(").push_bind(text).push("))"),
        ("reading_status", "equals") => { validate_status(&text)?; builder.push("COALESCE(s.reading_status, 'unread') = ").push_bind(text) },
        ("favorite", "is") => builder.push("COALESCE(s.favorite, false) = ").push_bind(rule.value.as_bool().unwrap_or(false)),
        ("library", "equals") => builder.push("b.library_id = ").push_bind(text.parse::<Uuid>().map_err(|_| AppError::BadRequest("library rule value must be a library id".to_string()))?),
        ("created_date", "after") => builder.push("b.created_at >= ").push_bind(text),
        ("created_date", "before") => builder.push("b.created_at <= ").push_bind(text),
        ("updated_date", "after") => builder.push("b.updated_at >= ").push_bind(text),
        ("updated_date", "before") => builder.push("b.updated_at <= ").push_bind(text),
        ("progress_percent", "greater_than") => builder.push("COALESCE(rp.progress_percent, 0) > ").push_bind(rule.value.as_f64().unwrap_or(0.0)),
        ("progress_percent", "less_than") => builder.push("COALESCE(rp.progress_percent, 0) < ").push_bind(rule.value.as_f64().unwrap_or(0.0)),
        _ => return Err(AppError::BadRequest(format!("unsupported shelf rule: {} {}", rule.field, rule.operator))),
    };
    Ok(())
}

fn parse_rules(value: &serde_json::Value) -> AppResult<Vec<ShelfRule>> {
    serde_json::from_value(value.clone()).map_err(|err| AppError::BadRequest(format!("invalid shelf rules: {err}")))
}

fn validate_shelf_input(name: &str, match_mode: &str, rules: &[ShelfRule]) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("shelf name is required".to_string()));
    }
    if match_mode != "all" && match_mode != "any" {
        return Err(AppError::BadRequest("match_mode must be all or any".to_string()));
    }
    if rules.len() > 12 {
        return Err(AppError::BadRequest("a shelf can have up to 12 rules".to_string()));
    }
    for rule in rules {
        validate_rule(rule)?;
    }
    Ok(())
}

fn validate_rule(rule: &ShelfRule) -> AppResult<()> {
    let supported = matches!((rule.field.as_str(), rule.operator.as_str()),
        ("title", "contains" | "equals") |
        ("author", "contains" | "equals") |
        ("language", "equals") |
        ("year", "equals") |
        ("tag", "equals") |
        ("reading_status", "equals") |
        ("favorite", "is") |
        ("library", "equals") |
        ("created_date", "after" | "before") |
        ("updated_date", "after" | "before") |
        ("progress_percent", "greater_than" | "less_than")
    );
    if !supported {
        return Err(AppError::BadRequest(format!("unsupported shelf rule: {} {}", rule.field, rule.operator)));
    }
    if rule.field == "reading_status" {
        validate_status(rule.value.as_str().unwrap_or_default())?;
    }
    Ok(())
}

async fn group_counts(db: &PgPool, user_id: Uuid, group: &str) -> AppResult<Vec<(String, i64)>> {
    let rows: Vec<(String, i64)> = match group {
        "author" => sqlx::query_as("SELECT a AS value, count(*) AS count FROM books b CROSS JOIN unnest(b.authors) a WHERE a <> '' GROUP BY a ORDER BY lower(a) LIMIT 50").fetch_all(db).await?,
        "language" => sqlx::query_as("SELECT language AS value, count(*) AS count FROM books WHERE language IS NOT NULL AND language <> '' GROUP BY language ORDER BY lower(language) LIMIT 50").fetch_all(db).await?,
        "year" => sqlx::query_as("SELECT left(published_date, 4) AS value, count(*) AS count FROM books WHERE published_date IS NOT NULL AND length(published_date) >= 4 GROUP BY left(published_date, 4) ORDER BY left(published_date, 4) DESC LIMIT 50").fetch_all(db).await?,
        "tag" => sqlx::query_as("SELECT t.name AS value, count(*) AS count FROM tags t JOIN book_tags bt ON bt.tag_id = t.id WHERE t.user_id = $1 AND bt.user_id = $1 GROUP BY t.name ORDER BY lower(t.name) LIMIT 50").bind(user_id).fetch_all(db).await?,
        _ => Vec::new(),
    };
    Ok(rows)
}

async fn find_smart_shelf(db: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<SmartShelf> {
    sqlx::query_as::<_, SmartShelf>(
        "SELECT id, user_id, name, match_mode, rules, created_at, updated_at FROM smart_shelves WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)
}

async fn shelf_summary(db: &PgPool, user_id: Uuid, shelf: SmartShelf) -> AppResult<ShelfSummary> {
    let rules = parse_rules(&shelf.rules)?;
    let count = list_books(db, user_id, BookFilters { shelf_id: Some(shelf.id.to_string()), ..Default::default() }).await?.len() as i64;
    Ok(ShelfSummary { id: shelf.id.to_string(), name: shelf.name, kind: "custom".to_string(), count, group: Some("custom".to_string()), rules: Some(rules), match_mode: Some(shelf.match_mode) })
}
