use std::{io::Write, path::Path};

use async_graphql::dynamic::Schema;

use crate::errors::RecorderResult;

/// Readers see the previous complete schema or the new one, including on
/// Windows.
pub(crate) fn export_schema(schema: &Schema, path: &Path) -> RecorderResult<()> {
  let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
  std::fs::create_dir_all(parent)?;
  let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
  temporary.write_all(schema.sdl().as_bytes())?;
  temporary.flush()?;
  temporary.persist(path).map_err(|error| error.error)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use async_graphql::dynamic::{Field, FieldFuture, Object, TypeRef};

  use super::*;

  fn schema() -> Schema {
    Schema::build("Query", None, None)
      .register(Object::new("Query").field(Field::new("version", TypeRef::named_nn(TypeRef::STRING), |_| {
        FieldFuture::new(async { Ok(Some(async_graphql::Value::from("fixture"))) })
      })))
      .finish()
      .unwrap()
  }

  #[test]
  fn schema_export_replaces_existing_output_and_cleans_up_failed_publication() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("nested/schema.graphql");
    let schema = schema();
    export_schema(&schema, &path).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), schema.sdl());
    std::fs::write(&path, "previous complete SDL").unwrap();
    export_schema(&schema, &path).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), schema.sdl());
    assert_eq!(std::fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    let invalid = directory.path().join("destination-directory");
    std::fs::create_dir(&invalid).unwrap();
    assert!(export_schema(&schema, &invalid).is_err());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
  }
}
