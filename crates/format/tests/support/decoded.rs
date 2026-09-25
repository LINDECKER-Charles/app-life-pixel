use life_pixel_format::{
    AnimationData, DecodeError, FrameData, FrameKind, Payload, Rgba, TagData, apply_frame,
};

/// Decodes a whole payload back into the animation it encodes, applying every frame.
pub fn decode_animation(bytes: &[u8]) -> Result<AnimationData, DecodeError> {
    let payload = Payload::parse(bytes)?;
    Ok(AnimationData {
        width: payload.width(),
        height: payload.height(),
        palette: palette(&payload),
        title: payload.title().to_owned(),
        tags: tags(&payload),
        frames: frames(&payload)?,
    })
}

/// The kind of each frame of a payload, in order.
pub fn frame_kinds(bytes: &[u8]) -> Result<Vec<FrameKind>, DecodeError> {
    let payload = Payload::parse(bytes)?;
    Ok(payload.frames().map(|frame| frame.kind).collect())
}

fn palette(payload: &Payload<'_>) -> Vec<Rgba> {
    (0..=u8::MAX)
        .map_while(|index| payload.palette_entry(index))
        .collect()
}

fn tags(payload: &Payload<'_>) -> Vec<TagData> {
    (0..payload.tag_count())
        .filter_map(|index| payload.tag(index))
        .map(|tag| TagData {
            name: tag.name.to_owned(),
            first: tag.first,
            last: tag.last,
            loop_mode: tag.loop_mode,
        })
        .collect()
}

fn frames(payload: &Payload<'_>) -> Result<Vec<FrameData>, DecodeError> {
    let mut indices = vec![0; usize::from(payload.width()) * usize::from(payload.height())];
    payload
        .frames()
        .map(|frame| {
            apply_frame(&frame, &mut indices, payload.palette_len())?;
            Ok(FrameData {
                duration_ms: frame.duration_ms,
                indices: indices.clone(),
            })
        })
        .collect()
}
