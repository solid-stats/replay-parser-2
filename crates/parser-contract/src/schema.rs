use schemars::Schema;

use crate::artifact::ParseArtifact;

pub fn parse_artifact_schema() -> Schema {
    schemars::schema_for!(ParseArtifact)
}
