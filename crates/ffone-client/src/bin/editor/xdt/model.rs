use super::*;
use std::cmp::Ordering;

#[derive(Clone)]
pub(super) struct Table {
    pub pointer: String,
    pub label: String,
}
#[derive(Clone)]
pub(super) struct Change {
    pub pointer: String,
    pub before: Value,
    pub after: Value,
}
pub(super) struct Link {
    pub table: usize,
    pub row: usize,
    pub field: String,
    pub value: String,
    pub confirmed: bool,
    pub incoming: bool,
}
pub(super) fn read(path: &Path) -> Result<Value, String> {
    let value = ffone_client::xdt::from_slice(&fs::read(path).map_err(|e| e.to_string())?)?;
    validate_document(&value)?;
    Ok(value)
}
fn validate_document(value: &Value) -> Result<(), String> {
    if value["schema"] != "ffone.table-set.v1" {
        return Err("Unsupported TableData schema".into());
    }
    let tables = value["tables"].as_array().ok_or("Missing tables")?;
    let mut names = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for table in tables {
        let name = table["name"].as_str().ok_or("Missing table name")?;
        if !names.insert(name) {
            return Err("Duplicate table identity".into());
        }
        if let Some(key) = table.get("key") {
            let key = key.as_str().ok_or("Table key must be a string")?;
            if !keys.insert(key) {
                return Err("Duplicate table key".into());
            }
        }
        if !table["value"].is_object() {
            return Err(format!("{name}: value must be an object"));
        }
    }
    Ok(())
}
fn escaped(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}
fn walk(value: &Value, pointer: String, label: String, tables: &mut Vec<Table>) {
    match value {
        Value::Array(rows) if rows.is_empty() || rows.iter().all(Value::is_object) => {
            tables.push(Table { pointer, label });
        }
        Value::Object(fields) => {
            for (key, value) in fields {
                walk(
                    value,
                    format!("{pointer}/{}", escaped(key)),
                    format!("{label}/{key}"),
                    tables,
                );
            }
        }
        _ => {}
    }
}
impl XdtEditor {
    pub(super) fn discover(&mut self) {
        let selected = self.tables.get(self.table).map(|t| t.pointer.clone());
        self.tables.clear();
        if let Some(tables) = self.document["tables"].as_array() {
            for (i, table) in tables.iter().enumerate() {
                walk(
                    &table["value"],
                    format!("/tables/{i}/value"),
                    table["name"].as_str().unwrap_or("?").to_owned(),
                    &mut self.tables,
                );
            }
        }
        self.table = selected
            .and_then(|p| self.tables.iter().position(|t| t.pointer == p))
            .or_else(|| {
                self.tables
                    .iter()
                    .position(|t| t.label.ends_with("m_pNpcTable/m_pNpcData"))
            })
            .unwrap_or(0);
        self.reindex();
        self.refresh();
    }
    pub(super) fn reindex(&mut self) {
        self.rebuild_summaries();
        self.references = super::relations::collect(&self.document, &self.tables);
        self.rebuild_search_index();
        self.index.clear();
        self.primary_domains.clear();
        for (t, table) in self.tables.iter().enumerate() {
            let mut domain_fields = BTreeMap::<String, Vec<String>>::new();
            let rows = self
                .document
                .pointer(&table.pointer)
                .and_then(Value::as_array)
                .unwrap();
            for (r, row) in self
                .document
                .pointer(&table.pointer)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
            {
                let mut fields = Vec::new();
                numeric_fields(row, "", &mut fields);
                for (field, value) in fields {
                    if id_field(&field) {
                        domain_fields
                            .entry(field.clone())
                            .or_default()
                            .push(value.clone());
                        self.index
                            .entry((identity(&field), value))
                            .or_default()
                            .push((t, r));
                    }
                }
            }
            for (field, values) in domain_fields {
                if values.len() == rows.len()
                    && values.iter().collect::<BTreeSet<_>>().len() == values.len()
                {
                    self.primary_domains.insert((t, identity(&field)));
                }
            }
        }
    }
    pub(super) fn rebuild_links(&mut self) {
        self.links.clear();
        let Some(row) = self.row.and_then(|i| self.rows().get(i)).cloned() else {
            return;
        };
        for reference in &self.references {
            if reference.table == reference.target_table
                && reference.target_row == Some(reference.row)
            {
                continue;
            }
            if reference.table == self.table && Some(reference.row) == self.row {
                if let Some(target_row) = reference.target_row {
                    self.links.push(Link {
                        table: reference.target_table,
                        row: target_row,
                        field: reference.field.clone(),
                        value: reference.value.to_string(),
                        confirmed: true,
                        incoming: false,
                    });
                }
            }
            if reference.target_table == self.table && reference.target_row == self.row {
                self.links.push(Link {
                    table: reference.table,
                    row: reference.row,
                    field: reference.field.clone(),
                    value: reference.value.to_string(),
                    confirmed: true,
                    incoming: true,
                });
            }
        }
        let mut fields = Vec::new();
        numeric_fields(&row, "", &mut fields);
        let mut seen = BTreeSet::new();
        for (field, value) in fields {
            if !id_field(&field) {
                continue;
            }
            if value == "0" {
                continue;
            }
            if let Some(targets) = self.index.get(&(identity(&field), value.clone())) {
                for &(table, row) in targets {
                    let domain = identity(&field);
                    if !self.primary_domains.contains(&(self.table, domain.clone()))
                        && !self.primary_domains.contains(&(table, domain))
                    {
                        continue;
                    }
                    if Some(row) == self.row && table == self.table {
                        continue;
                    }
                    if seen.insert((field.clone(), table, row)) {
                        if !self
                            .links
                            .iter()
                            .any(|l| l.confirmed && l.table == table && l.row == row)
                        {
                            self.links.push(Link {
                                table,
                                row,
                                field: field.clone(),
                                value: value.clone(),
                                confirmed: false,
                                incoming: false,
                            });
                        }
                    }
                }
            }
        }
        self.links.sort_by_key(|link| {
            (
                !link.confirmed,
                link.incoming,
                link.field.clone(),
                link.table,
                link.row,
            )
        });
    }
}
// Relations are labelled candidates: equality of an ID domain is not a foreign-key declaration.
fn identity(field: &str) -> String {
    let field = field
        .rsplit('/')
        .find(|s| s.parse::<usize>().is_err())
        .unwrap_or(field);
    let field = field
        .strip_prefix("m_i")
        .or_else(|| field.strip_prefix("m_u"))
        .unwrap_or(field);
    let lower = field.to_lowercase();
    let lower = lower.trim_end_matches(|c: char| c.is_ascii_digit());
    let lower = lower
        .strip_suffix("number")
        .or_else(|| lower.strip_suffix("id"))
        .unwrap_or(lower);
    lower.to_owned()
}
pub(super) fn id_field(field: &str) -> bool {
    let field = field
        .rsplit('/')
        .find(|s| s.parse::<usize>().is_err())
        .unwrap_or(field);
    let field = field
        .strip_prefix("m_i")
        .or_else(|| field.strip_prefix("m_u"))
        .unwrap_or(field);
    let lower = field.to_lowercase();
    let lower = lower.trim_end_matches(|c: char| c.is_ascii_digit());
    lower.ends_with("id") || lower.ends_with("_id") || lower.ends_with("number") || lower == "icon"
}
fn numeric_fields(value: &Value, prefix: &str, result: &mut Vec<(String, String)>) {
    match value {
        Value::String(value) if !value.is_empty() => {
            result.push((prefix.to_owned(), value.clone()))
        }
        Value::Number(n) if n.is_i64() || n.is_u64() => {
            result.push((prefix.to_owned(), n.to_string()))
        }
        Value::Object(fields) => {
            for (key, value) in fields {
                numeric_fields(value, &format!("{prefix}/{key}"), result);
            }
        }
        Value::Array(items) => {
            for (i, value) in items.iter().enumerate() {
                numeric_fields(value, &format!("{prefix}/{i}"), result);
            }
        }
        _ => {}
    }
}
pub(super) fn display(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}
pub(super) fn compare(a: &Value, b: &Value) -> Ordering {
    match (a.as_i64(), b.as_i64(), a.as_u64(), b.as_u64()) {
        (Some(a), Some(b), _, _) => a.cmp(&b),
        (_, _, Some(a), Some(b)) => a.cmp(&b),
        _ => match (a.as_f64(), b.as_f64()) {
            (Some(a), Some(b)) => a.total_cmp(&b),
            _ => display(a).to_lowercase().cmp(&display(b).to_lowercase()),
        },
    }
}
pub(super) fn parse_cell(old: &Value, text: &str) -> Result<Value, String> {
    if old.is_string() {
        return Ok(Value::String(text.to_owned()));
    }
    let value: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let same = match old {
        Value::Null => true,
        Value::Bool(_) => value.is_boolean(),
        Value::Number(n) if n.is_i64() => value.as_i64().is_some(),
        Value::Number(n) if n.is_u64() => value.as_u64().is_some(),
        Value::Number(_) => value.is_number(),
        Value::Array(_) => value.is_array(),
        Value::Object(_) => value.is_object(),
        _ => false,
    };
    if !same {
        return Err("Value type must match the original field".into());
    }
    Ok(value)
}
pub(super) fn validate_rows(before: &[Value], after: &[Value]) -> Result<(), String> {
    if after.iter().any(|r| !r.is_object()) {
        return Err("Rows must be JSON objects".into());
    }
    let mut types = BTreeMap::new();
    for row in before.iter().filter_map(Value::as_object) {
        for (field, value) in row {
            let examples: &mut Vec<&Value> = types.entry(field).or_default();
            let kind = |v: &Value| match v {
                Value::Null => 0,
                Value::Bool(_) => 1,
                Value::Number(n) if n.is_i64() => 2,
                Value::Number(n) if n.is_u64() => 3,
                Value::Number(_) => 4,
                Value::String(_) => 5,
                Value::Array(_) => 6,
                Value::Object(_) => 7,
            };
            if !examples.iter().any(|old| kind(old) == kind(value)) {
                examples.push(value);
            }
        }
    }
    for row in after {
        for (field, value) in row.as_object().into_iter().flatten() {
            if let Some(examples) = types.get(field) {
                if !examples.iter().any(|old| {
                    parse_cell(old, &display(value)).is_ok_and(|parsed| parsed == *value)
                }) {
                    return Err(schema::task_error(row, format!("Wrong value type: {field}")));
                }
            }
        }
    }
    // Mission IDs group stages; NPC/journal IDs are references and may repeat.
    // Only the task ID identifies a mission row, even in a one-stage table.
    let mission_rows = before.iter().any(|row| row.get("m_iHTaskID").is_some());
    // Preserve unique, populated identity columns; accepted duplicates in the source are retained.
    let fields: BTreeSet<_> = before
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|r| {
            r.keys()
                .filter(|k| id_field(k) && !k.to_lowercase().contains("icon")
                    && (!mission_rows || k.as_str() == "m_iHTaskID"))
                .cloned()
        })
        .collect();
    for field in fields {
        let old: Vec<_> = before.iter().filter_map(|r| r.get(&field)).collect();
        if old.len() != before.len() || old.is_empty() {
            continue;
        }
        let unique: BTreeSet<_> = old.iter().map(|v| v.to_string()).collect();
        if unique.len() != old.len() {
            continue;
        }
        let mut ids = BTreeSet::new();
        for row in after {
            let value = row
                .get(&field)
                .ok_or_else(|| schema::task_error(row, format!("Missing identity field: {field}")))?;
            parse_cell(old[0], &display(value)).map_err(|error| schema::task_error(row, error))?;
            if !ids.insert(value.to_string()) {
                return Err(schema::task_error(row, format!("Duplicate identity: {field} = {value}")));
            }
        }
    }
    Ok(())
}
// Three-way merge at field level. Arrays conflict as a unit when their structure changes.
pub(super) fn merge_document(base: &Value, draft: &Value, disk: &Value) -> Result<Value, String> {
    fn merge(b: &Value, d: &Value, s: &Value, path: &str) -> Result<Value, String> {
        if b == d {
            return Ok(s.clone());
        }
        if b == s || d == s {
            return Ok(d.clone());
        }
        if let (Some(b), Some(d), Some(s)) = (b.as_object(), d.as_object(), s.as_object()) {
            let mut result = s.clone();
            for key in b.keys().chain(d.keys()).collect::<BTreeSet<_>>() {
                if b.get(key) == d.get(key) {
                    continue;
                }
                match (b.get(key), d.get(key), s.get(key)) {
                    (Some(b), Some(d), Some(s)) => {
                        result.insert(key.clone(), merge(b, d, s, &format!("{path}/{key}"))?);
                    }
                    (_, d, s) if s == b.get(key) || s == d => {
                        if let Some(d) = d {
                            result.insert(key.clone(), d.clone());
                        } else {
                            result.remove(key);
                        }
                    }
                    _ => return Err(format!("External change conflict: {path}/{key}")),
                }
            }
            return Ok(Value::Object(result));
        }
        if let (Some(b), Some(d), Some(s)) = (b.as_array(), d.as_array(), s.as_array()) {
            if b.len() == d.len() && b.len() == s.len() {
                let fields: BTreeSet<_> = b
                    .iter()
                    .filter_map(Value::as_object)
                    .flat_map(|row| row.keys())
                    .filter(|field| {
                        id_field(field) || (path == "/tables" && field.as_str() == "name")
                    })
                    .collect();
                for field in fields {
                    let identities: Vec<_> = b.iter().filter_map(|row| row.get(field)).collect();
                    if identities.len() != b.len()
                        || identities
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<BTreeSet<_>>()
                            .len()
                            != identities.len()
                    {
                        continue;
                    }
                    if b.iter().zip(d).any(|(b, d)| b.get(field) != d.get(field))
                        || b.iter().zip(s).any(|(b, s)| b.get(field) != s.get(field))
                    {
                        return Err(format!(
                            "External change conflict: {path} (row identities/order changed)"
                        ));
                    }
                }
                return b
                    .iter()
                    .zip(d)
                    .zip(s)
                    .enumerate()
                    .map(|(i, ((b, d), s))| merge(b, d, s, &format!("{path}/{i}")))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Value::Array);
            }
        }
        Err(format!("External change conflict: {path}"))
    }
    let merged = merge(base, draft, disk, "")?;
    validate_document(&merged)?;
    // Check merged uniqueness too (two individually valid writers can create the same ID).
    let mut tables = Vec::new();
    for (i, table) in merged["tables"].as_array().unwrap().iter().enumerate() {
        walk(
            &table["value"],
            format!("/tables/{i}/value"),
            String::new(),
            &mut tables,
        );
    }
    for table in tables {
        if let (Some(b), Some(a)) = (
            base.pointer(&table.pointer).and_then(Value::as_array),
            merged.pointer(&table.pointer).and_then(Value::as_array),
        ) {
            validate_rows(b, a)?;
        }
    }
    Ok(merged)
}
