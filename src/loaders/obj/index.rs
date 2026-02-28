pub type FaceVertex = (usize, Option<usize>, Option<usize>);

pub fn parse_f32_component(raw: &str, line_number: usize, label: &str) -> Result<f32, String> {
    raw.parse::<f32>().map_err(|error| {
        format!(
            "OBJ line {}: invalid {} '{}': {}",
            line_number, label, raw, error
        )
    })
}

pub fn parse_face_vertex(
    token: &str,
    line_number: usize,
    positions_len: usize,
    texcoords_len: usize,
    normals_len: usize,
) -> Result<FaceVertex, String> {
    let fields: Vec<&str> = token.split('/').collect();
    if fields.is_empty() || fields.len() > 3 {
        return Err(format!(
            "OBJ line {}: invalid face vertex token '{}'",
            line_number, token
        ));
    }

    if fields[0].is_empty() {
        return Err(format!(
            "OBJ line {}: missing vertex position index in face token '{}'",
            line_number, token
        ));
    }

    let position_index = parse_obj_index(fields[0], positions_len, line_number, "position")?;

    let texcoord_index = if fields.len() > 1 && !fields[1].is_empty() {
        Some(parse_obj_index(
            fields[1],
            texcoords_len,
            line_number,
            "texcoord",
        )?)
    } else {
        None
    };

    let normal_index = if fields.len() > 2 && !fields[2].is_empty() {
        Some(parse_obj_index(
            fields[2],
            normals_len,
            line_number,
            "normal",
        )?)
    } else {
        None
    };

    Ok((position_index, texcoord_index, normal_index))
}

fn parse_obj_index(
    raw: &str,
    count: usize,
    line_number: usize,
    label: &str,
) -> Result<usize, String> {
    let parsed = raw.parse::<isize>().map_err(|error| {
        format!(
            "OBJ line {}: invalid {} index '{}': {}",
            line_number, label, raw, error
        )
    })?;

    if parsed == 0 {
        return Err(format!(
            "OBJ line {}: {} index 0 is invalid in OBJ format",
            line_number, label
        ));
    }

    if count == 0 {
        return Err(format!(
            "OBJ line {}: {} index '{}' referenced before any {} data was defined",
            line_number, label, raw, label
        ));
    }

    let resolved = if parsed > 0 {
        parsed - 1
    } else {
        count as isize + parsed
    };

    if resolved < 0 || resolved as usize >= count {
        return Err(format!(
            "OBJ line {}: {} index '{}' is out of bounds (count={})",
            line_number, label, raw, count
        ));
    }

    Ok(resolved as usize)
}

pub fn directive_value<'a>(
    line: &'a str,
    directive: &str,
    line_number: usize,
) -> Result<&'a str, String> {
    let value = line
        .strip_prefix(directive)
        .map(str::trim)
        .filter(|s| !s.is_empty());

    value.ok_or_else(|| {
        format!(
            "Line {}: directive '{}' is missing a required value",
            line_number, directive
        )
    })
}
