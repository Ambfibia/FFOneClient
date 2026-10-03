use super::*;

pub(super) fn find_combining_arrays<'a>(value: &'a Value, candidates: &mut Vec<&'a Vec<Value>>) {
    match value {
        Value::Object(object) => {
            if let Some(Value::Object(table)) = object.get("m_pCombiningTable")
                && let Some(Value::Array(rows)) = table.get("m_pCombiningData")
            {
                candidates.push(rows);
            }
            for nested in object.values() {
                find_combining_arrays(nested, candidates);
            }
        }
        Value::Array(values) => {
            for nested in values {
                find_combining_arrays(nested, candidates);
            }
        }
        _ => {}
    }
}

pub(super) fn parse_recipe_row(
    index: usize,
    value: &Value,
) -> Result<CombiRecipe0104, CombiRecipeTableError0104> {
    let object = value
        .as_object()
        .ok_or(CombiRecipeTableError0104::RowNotObject { index })?;
    Ok(CombiRecipe0104 {
        level_gap: recipe_i32(object, index, "m_iLevelGap")?,
        level_gap_standard: recipe_f32(object, index, "m_fLevelGapStandard")?,
        same_grade: recipe_f32(object, index, "m_fSameGrade")?,
        one_grade: recipe_f32(object, index, "m_fOneGrade")?,
        two_grade: recipe_f32(object, index, "m_fTwoGrade")?,
        three_grade: recipe_f32(object, index, "m_fThreeGrade")?,
        look_constant: recipe_i32(object, index, "m_iLookConstant")?,
        stat_constant: recipe_i32(object, index, "m_iStatConstant")?,
    })
}
