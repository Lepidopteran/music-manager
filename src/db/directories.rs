use axum::response::IntoResponse;
use hyper::StatusCode;

use super::{Directory, NewDirectory, Result, Connection};

#[derive(thiserror::Error, Debug)]
pub enum DatabaseDirectoryError {
    #[error("Directory not found")]
    NotFound,
    #[error("Name is empty")]
    NameEmpty,
    #[error("Path is empty")]
    PathEmpty,
    #[error("Path \"{0}\" does not exist")]
    PathDoesNotExist(String),
    #[error("Path \"{0}\" is not a directory")]
    PathNotDirectory(String),
    #[error("Path \"{0}\" is not absolute")]
    PathNotAbsolute(String),
    #[error("Path \"{0}\" is a subdirectory of an existing directory")]
    PathIsSubdirectory(String),
    #[error("Directory already exists")]
    PathAlreadyAdded,
    #[error("Path is not a valid UTF-8 string")]
    PathNotUtf8,
}

impl IntoResponse for DatabaseDirectoryError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            Self::PathAlreadyAdded | Self::PathIsSubdirectory(_) => {
                (StatusCode::CONFLICT, self.to_string()).into_response()
            }
            _ => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
        }
    }
}

pub async fn add_directory(
    connection: &mut Connection,
    sub_path: &str,
    directory: NewDirectory,
) -> Result<Directory> {
    if directory.path.trim().is_empty() {
        return Err(DatabaseDirectoryError::PathEmpty.into());
    }

    if std::str::from_utf8(directory.path.as_bytes()).is_err() {
        return Err(DatabaseDirectoryError::PathNotUtf8.into());
    }

    if sqlx::query_as!(
        Directory,
        "SELECT * FROM directories WHERE path = ?",
        directory.path
    )
    .fetch_optional(&mut *connection)
    .await?
    .is_some()
    {
        return Err(DatabaseDirectoryError::PathAlreadyAdded.into());
    }

    let path = std::path::Path::new(&directory.path);

    if !path.is_absolute() {
        return Err(DatabaseDirectoryError::PathNotAbsolute(directory.path).into());
    }

    if !path.exists() {
        return Err(DatabaseDirectoryError::PathDoesNotExist(directory.path).into());
    }

    if !path.is_dir() {
        return Err(DatabaseDirectoryError::PathNotDirectory(directory.path).into());
    }

    let directories = sqlx::query_scalar!("SELECT path FROM directories")
        .fetch_all(&mut *connection)
        .await?;

    for entry in directories {
        if path.starts_with(entry) {
            return Err(DatabaseDirectoryError::PathIsSubdirectory(directory.path).into());
        }
    }

    let uuid = uuid::Uuid::new_v4().to_string();

    let _ = sqlx::query!(
        "INSERT INTO directories (name, path, display_name) VALUES (?, ?, ?)",
        uuid,
        directory.path,
        directory.display_name
    )
    .execute(&mut *connection)
    .await?;

    Ok(Directory {
        name: uuid,
        path: directory.path,
        display_name: directory.display_name,
    })
}

pub async fn remove_directory(
    connection: &mut Connection,
    name: String,
) -> Result<()> {
    if name.trim().is_empty() {
        return Err(DatabaseDirectoryError::NameEmpty.into());
    }

    sqlx::query!("DELETE FROM songs WHERE directory_id = ?", name)
        .execute(&mut *connection)
        .await?;

    let rows_affected = sqlx::query!("DELETE FROM directories WHERE name = ?", name)
        .execute(&mut *connection)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        Err(DatabaseDirectoryError::NotFound.into())
    } else {
        Ok(())
    }
}

pub async fn get_directories(
    connection: &mut Connection,
) -> Result<Vec<Directory>> {
    let directories = sqlx::query_as!(Directory, "SELECT * FROM directories")
        .fetch_all(&mut *connection)
        .await?;

    Ok(directories)
}

pub async fn get_directory<'c>(
    connection: impl sqlx::Acquire<'c, Database = sqlx::Sqlite>,
    name: &str,
) -> Result<Directory> {
    let mut connection = connection.acquire().await?;
    sqlx::query_as!(Directory, "SELECT * FROM directories WHERE name = ?", name)
        .fetch_one(&mut *connection)
        .await
        .map_err(Into::into)
}
