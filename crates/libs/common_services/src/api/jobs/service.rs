use crate::api::app_error::AppError;
use crate::api::jobs::interfaces::{JobInfo, JobsQuery, PaginatedJobsResponse};
use sqlx::{PgPool, Postgres, QueryBuilder};

/// Maps safe API fields (either camelCase or snake_case) to valid DB columns.
/// Serves as an allowlist against SQL injection.
fn map_field_to_column(field: &str) -> Option<&'static str> {
    match field {
        "id" => Some("id"),
        "relative_path" | "relativePath" => Some("relative_path"),
        "user_id" | "userId" => Some("user_id"),
        "job_type" | "jobType" => Some("job_type"),
        "payload" => Some("payload"),
        "priority" => Some("priority"),
        "status" => Some("status"),
        "attempts" => Some("attempts"),
        "dependency_attempts" | "dependencyAttempts" => Some("dependency_attempts"),
        "max_attempts" | "maxAttempts" => Some("max_attempts"),
        "owner" => Some("owner"),
        "started_at" | "startedAt" => Some("started_at"),
        "finished_at" | "finishedAt" => Some("finished_at"),
        "created_at" | "createdAt" => Some("created_at"),
        "scheduled_at" | "scheduledAt" => Some("scheduled_at"),
        "last_heartbeat" | "lastHeartbeat" => Some("last_heartbeat"),
        "last_error" | "lastError" => Some("last_error"),
        _ => None,
    }
}

/// Provides correct SQL types for Postgres type casting on parameter binds.
fn get_column_cast_suffix(column: &str) -> &'static str {
    match column {
        "id" => "::bigint",
        "relative_path" => "::text",
        "user_id" => "::integer",
        "job_type" => "::job_type",
        "payload" => "::jsonb",
        "priority" => "::integer",
        "status" => "::job_status",
        "attempts" => "::integer",
        "dependency_attempts" => "::integer",
        "max_attempts" => "::integer",
        "owner" => "::text",
        "started_at" => "::timestamp with time zone",
        "finished_at" => "::timestamp with time zone",
        "created_at" => "::timestamp with time zone",
        "scheduled_at" => "::timestamp with time zone",
        "last_heartbeat" => "::timestamp with time zone",
        "last_error" => "::text",
        _ => "",
    }
}

#[derive(Debug, Clone)]
struct Filter {
    column: &'static str,
    operator: &'static str,
    value: Option<String>,
}

fn parse_filter(filter_str: &str) -> Result<Filter, AppError> {
    let parts: Vec<&str> = filter_str.splitn(3, ':').collect();
    if parts.is_empty() {
        return Err(AppError::BadRequest("Filter cannot be empty".to_owned()));
    }

    let raw_field = parts[0];
    let column = map_field_to_column(raw_field)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid filter field: {raw_field}")))?;

    if parts.len() == 1 {
        return Err(AppError::BadRequest(format!(
            "Filter '{filter_str}' is missing an operator"
        )));
    }

    let raw_op = parts[1].to_lowercase();
    let (operator, needs_value) = match raw_op.as_str() {
        "eq" | "equals" | "==" => ("=", true),
        "neq" | "notequals" | "!=" => ("!=", true),
        "gt" | ">" => (">", true),
        "gte" | ">=" => (">=", true),
        "lt" | "<" => ("<", true),
        "lte" | "<=" => ("<=", true),
        "contains" | "like" => ("ILIKE", true),
        "isnull" | "is_null" => ("IS NULL", false),
        "isnotnull" | "is_not_null" => ("IS NOT NULL", false),
        _ => {
            return Err(AppError::BadRequest(format!(
                "Invalid filter operator: {raw_op}"
            )))
        }
    };

    let value = if needs_value {
        if parts.len() < 3 {
            return Err(AppError::BadRequest(format!(
                "Filter '{filter_str}' is missing a value"
            )));
        }
        let mut val = parts[2].to_string();
        if operator == "ILIKE" {
            val = format!("%{}%", val);
        }
        Some(val)
    } else {
        None
    };

    Ok(Filter {
        column,
        operator,
        value,
    })
}

#[derive(Debug, Clone)]
struct Sort {
    column: &'static str,
    direction: &'static str,
}

fn parse_sort(sort_str: &str) -> Result<Sort, AppError> {
    let parts: Vec<&str> = sort_str.splitn(2, ':').collect();
    if parts.is_empty() {
        return Err(AppError::BadRequest("Sort cannot be empty".to_owned()));
    }

    let raw_field = parts[0];
    let column = map_field_to_column(raw_field)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid sort field: {raw_field}")))?;

    let direction = if parts.len() == 2 {
        let dir = parts[1].to_lowercase();
        if dir == "desc" || dir == "descending" {
            "DESC"
        } else if dir == "asc" || dir == "ascending" {
            "ASC"
        } else {
            return Err(AppError::BadRequest(format!(
                "Invalid sort direction '{dir}'. Must be 'asc' or 'desc'"
            )));
        }
    } else {
        "ASC"
    };

    Ok(Sort {
        column,
        direction,
    })
}

fn apply_filters<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    filters: &[Filter],
) {
    if !filters.is_empty() {
        builder.push(" WHERE ");
        for (i, filter) in filters.iter().enumerate() {
            if i > 0 {
                builder.push(" AND ");
            }
            builder.push(filter.column);
            if let Some(ref val) = filter.value {
                builder.push(" ");
                builder.push(filter.operator);
                builder.push(" ");
                let cast_suffix = get_column_cast_suffix(filter.column);
                builder.push_bind(val.clone());
                builder.push(cast_suffix);
            } else {
                builder.push(" ");
                builder.push(filter.operator);
            }
        }
    }
}

pub async fn get_job_overview(
    pool: &PgPool,
    query: JobsQuery,
) -> Result<PaginatedJobsResponse, AppError> {
    // 1. Parse and extract sorts
    let mut sorts = Vec::new();
    for s_str in &query.sort {
        for part in s_str.split(',') {
            let part_trimmed = part.trim();
            if !part_trimmed.is_empty() {
                sorts.push(parse_sort(part_trimmed)?);
            }
        }
    }
    if sorts.is_empty() {
        sorts.push(Sort {
            column: "id",
            direction: "DESC",
        });
    }

    // 2. Parse and validate filters
    let mut filters = Vec::new();
    for f_str in &query.filter {
        let f_trimmed = f_str.trim();
        if !f_trimmed.is_empty() {
            filters.push(parse_filter(f_trimmed)?);
        }
    }

    // 3. Paginate calculations
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = if let Some(off) = query.offset {
        off.max(0)
    } else if let Some(pg) = query.page {
        ((pg - 1).max(0)) * limit
    } else {
        0
    };

    // 4. Build and execute total count query
    let mut count_builder = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM jobs");
    apply_filters(&mut count_builder, &filters);

    let count_query = count_builder.build_query_scalar::<i64>();
    let total = count_query.fetch_one(pool).await?;

    // 5. Build select query
    let mut select_builder = QueryBuilder::<Postgres>::new(
        "SELECT id, relative_path, user_id, job_type, payload, priority, status, attempts, \
         dependency_attempts, max_attempts, owner, started_at, finished_at, created_at, \
         scheduled_at, last_heartbeat, last_error FROM jobs"
    );
    apply_filters(&mut select_builder, &filters);

    // Apply multi-level sort
    select_builder.push(" ORDER BY ");
    for (i, sort) in sorts.iter().enumerate() {
        if i > 0 {
            select_builder.push(", ");
        }
        select_builder.push(sort.column);
        select_builder.push(" ");
        select_builder.push(sort.direction);
    }

    // Apply pagination limits
    select_builder.push(" LIMIT ");
    select_builder.push_bind(limit);
    select_builder.push(" OFFSET ");
    select_builder.push_bind(offset);

    // Execute select query
    let select_query = select_builder.build_query_as::<JobInfo>();
    let data = select_query.fetch_all(pool).await?;

    Ok(PaginatedJobsResponse {
        data,
        total,
        limit,
        offset,
    })
}