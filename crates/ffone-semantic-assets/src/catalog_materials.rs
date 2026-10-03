use super::*;

pub(super) fn render_route(asset_root: &Path, route: &SemanticRoute, staged_path: &Path) -> Result<()> {
    match &route.content {
        RouteContent::CopyExact => {
            let source = asset_root.join(native_path(&route.source_path));
            fs::copy(&source, staged_path).map_err(|source| io_at(staged_path, source))?;
        }
        RouteContent::RewriteJson { replacements } => {
            let source = asset_root.join(native_path(&route.source_path));
            let mut value = read_json(&source)?;
            rewrite_json_strings(&mut value, replacements);
            write_json(staged_path, &value)?;
        }
        RouteContent::GeneratedJson { value } => write_json(staged_path, value)?,
    }
    Ok(())
}
