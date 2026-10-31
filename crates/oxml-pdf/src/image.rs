//! Image embedding for PDF output (JPEG pass-through, PNG decompression).

use std::collections::HashMap;

use oxml_media::{ImageFormat, probe};

/// Decoded image data ready for PDF embedding.
pub(crate) struct DecodedImage {
    /// Raw pixel data (RGB or grayscale) or JPEG bytes for pass-through.
    pub data: Vec<u8>,
    /// Alpha channel data (if present), for a soft mask.
    pub alpha: Option<Vec<u8>>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Color space: "DeviceRGB", "DeviceGray".
    pub color_space: &'static str,
    /// Whether data is raw JPEG (pass through with DCTDecode).
    pub is_jpeg: bool,
}

/// Decode image bytes into a format suitable for PDF embedding.
///
/// JPEG images are passed through directly (the PDF viewer decodes them).
/// PNG images are decoded to raw RGB/RGBA pixels.
pub(crate) fn decode_image(data: &[u8], content_type: &str) -> Option<DecodedImage> {
    if content_type.contains("jpeg")
        || content_type.contains("jpg")
        || ImageFormat::sniff(data) == Some(ImageFormat::Jpeg)
    {
        decode_jpeg(data)
    } else if ImageFormat::sniff(data) == Some(ImageFormat::Emf) {
        decode_emf(data)
    } else {
        decode_png_or_other(data)
    }
}

fn decode_jpeg(data: &[u8]) -> Option<DecodedImage> {
    let info = probe(data)?;
    if info.format != ImageFormat::Jpeg {
        return None;
    }

    Some(DecodedImage {
        data: data.to_vec(),
        alpha: None,
        width: info.width_px,
        height: info.height_px,
        color_space: "DeviceRGB",
        is_jpeg: true,
    })
}

fn decode_png_or_other(data: &[u8]) -> Option<DecodedImage> {
    // Use a minimal PNG decoder. We need to decode to raw pixels.
    // For simplicity, we'll parse the PNG ourselves for common cases,
    // or fall back to a basic RGBA decode.
    decode_png(data)
}

/// Minimal PNG decoding to raw RGB + optional alpha.
fn decode_png(data: &[u8]) -> Option<DecodedImage> {
    // Validate PNG signature
    if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }

    let mut pos = 8;
    let mut width = 0u32;
    let mut height = 0u32;
    let mut bit_depth = 0u8;
    let mut color_type = 0u8;
    let mut interlace = 0u8;
    let mut idat_data = Vec::new();
    // Palette entries as RGB triples, for indexed images.
    let mut palette: Vec<u8> = Vec::new();
    // Per-palette-entry alpha. Only meaningful for indexed images, where tRNS
    // is a list of alpha bytes indexed the same way as PLTE.
    let mut palette_alpha: Vec<u8> = Vec::new();

    while pos + 8 <= data.len() {
        let chunk_len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let chunk_type = &data[pos + 4..pos + 8];
        let chunk_data_start = pos + 8;
        let chunk_data_end = chunk_data_start + chunk_len;

        if chunk_data_end > data.len() {
            break;
        }

        match chunk_type {
            b"IHDR" => {
                if chunk_len >= 13 {
                    let d = &data[chunk_data_start..];
                    width = u32::from_be_bytes([d[0], d[1], d[2], d[3]]);
                    height = u32::from_be_bytes([d[4], d[5], d[6], d[7]]);
                    bit_depth = d[8];
                    color_type = d[9];
                    interlace = d[12];
                }
            }
            b"IDAT" => {
                idat_data.extend_from_slice(&data[chunk_data_start..chunk_data_end]);
            }
            b"PLTE" => {
                palette.clear();
                palette.extend_from_slice(&data[chunk_data_start..chunk_data_end]);
            }
            b"tRNS" => {
                palette_alpha.clear();
                palette_alpha.extend_from_slice(&data[chunk_data_start..chunk_data_end]);
            }
            b"IEND" => break,
            _ => {}
        }

        pos = chunk_data_end + 4; // +4 for CRC
    }

    if width == 0 || height == 0 || idat_data.is_empty() || bit_depth != 8 {
        return None;
    }

    // Decompress the IDAT data (zlib deflate)
    let decompressed = miniz_oxide::inflate::decompress_to_vec_zlib(&idat_data).ok()?;

    // Unfilter the scanlines
    let channels: usize = match color_type {
        0 => 1, // Grayscale
        2 => 3, // RGB
        3 => 1, // Indexed: one palette index per pixel
        4 => 2, // Grayscale + Alpha
        6 => 4, // RGBA
        _ => return None,
    };

    let unfiltered = match interlace {
        0 => unfilter_scanlines(&decompressed, width as usize, height as usize, channels)?.0,
        1 => deinterlace_adam7(&decompressed, width as usize, height as usize, channels)?,
        _ => return None,
    };

    // Separate color and alpha channels
    match color_type {
        0 => {
            // Grayscale
            Some(DecodedImage {
                data: unfiltered,
                alpha: None,
                width,
                height,
                color_space: "DeviceGray",
                is_jpeg: false,
            })
        }
        2 => {
            // RGB
            Some(DecodedImage {
                data: unfiltered,
                alpha: None,
                width,
                height,
                color_space: "DeviceRGB",
                is_jpeg: false,
            })
        }
        3 => {
            // Indexed colour: every byte is an index into the palette.
            // Word embeds these routinely, because most tools default to a
            // palette when an image has few colours.
            if palette.len() < 3 {
                return None;
            }
            let entries = palette.len() / 3;
            let pixel_count = (width * height) as usize;
            let mut rgb = Vec::with_capacity(pixel_count * 3);
            let mut alpha = Vec::with_capacity(pixel_count);
            let mut all_opaque = true;
            for &index in unfiltered.iter().take(pixel_count) {
                let i = index as usize;
                if i < entries {
                    rgb.extend_from_slice(&palette[i * 3..i * 3 + 3]);
                } else {
                    // An index past the end of the palette is malformed. Emit
                    // black rather than dropping the whole image.
                    rgb.extend_from_slice(&[0, 0, 0]);
                }
                // tRNS may cover only the first few entries. Anything it does
                // not mention is opaque.
                let a = palette_alpha.get(i).copied().unwrap_or(255);
                if a != 255 {
                    all_opaque = false;
                }
                alpha.push(a);
            }
            Some(DecodedImage {
                data: rgb,
                // Match the RGBA path: no SMask when nothing is transparent.
                alpha: if all_opaque { None } else { Some(alpha) },
                width,
                height,
                color_space: "DeviceRGB",
                is_jpeg: false,
            })
        }
        4 => {
            // Grayscale + Alpha
            let pixel_count = (width * height) as usize;
            let mut gray = Vec::with_capacity(pixel_count);
            let mut alpha = Vec::with_capacity(pixel_count);
            for i in 0..pixel_count {
                gray.push(unfiltered[i * 2]);
                alpha.push(unfiltered[i * 2 + 1]);
            }
            Some(DecodedImage {
                data: gray,
                alpha: Some(alpha),
                width,
                height,
                color_space: "DeviceGray",
                is_jpeg: false,
            })
        }
        6 => {
            // RGBA
            let pixel_count = (width * height) as usize;
            let mut rgb = Vec::with_capacity(pixel_count * 3);
            let mut alpha = Vec::with_capacity(pixel_count);
            let mut all_opaque = true;
            for i in 0..pixel_count {
                rgb.push(unfiltered[i * 4]);
                rgb.push(unfiltered[i * 4 + 1]);
                rgb.push(unfiltered[i * 4 + 2]);
                let a = unfiltered[i * 4 + 3];
                alpha.push(a);
                if a != 255 {
                    all_opaque = false;
                }
            }
            // Skip alpha channel if fully opaque — avoids unnecessary SMask
            // that can cause subtle color rendering differences in PDF viewers
            Some(DecodedImage {
                data: rgb,
                alpha: if all_opaque { None } else { Some(alpha) },
                width,
                height,
                color_space: "DeviceRGB",
                is_jpeg: false,
            })
        }
        _ => None,
    }
}

/// The seven Adam7 passes: first column, first row, column step, row step.
const ADAM7_PASSES: [(usize, usize, usize, usize); 7] = [
    (0, 0, 8, 8),
    (4, 0, 8, 8),
    (0, 4, 4, 8),
    (2, 0, 4, 4),
    (0, 2, 2, 4),
    (1, 0, 2, 2),
    (0, 1, 1, 2),
];

/// Reassembles an interlaced (Adam7) image: each pass is a filtered
/// sub-image of its own, and its pixels are scattered to their positions.
fn deinterlace_adam7(data: &[u8], width: usize, height: usize, channels: usize) -> Option<Vec<u8>> {
    let mut image = vec![0u8; width.checked_mul(height)?.checked_mul(channels)?];
    let mut offset = 0;
    for (x0, y0, dx, dy) in ADAM7_PASSES {
        let pass_width = width.saturating_sub(x0).div_ceil(dx);
        let pass_height = height.saturating_sub(y0).div_ceil(dy);
        if pass_width == 0 || pass_height == 0 {
            continue;
        }
        let (pixels, consumed) =
            unfilter_scanlines(data.get(offset..)?, pass_width, pass_height, channels)?;
        offset += consumed;
        for row in 0..pass_height {
            for column in 0..pass_width {
                let from = (row * pass_width + column) * channels;
                let to = ((y0 + row * dy) * width + x0 + column * dx) * channels;
                image[to..to + channels].copy_from_slice(&pixels[from..from + channels]);
            }
        }
    }
    Some(image)
}

/// Reverses the per-row PNG filters of one image or pass, returning the
/// pixels and the number of input bytes consumed.
fn unfilter_scanlines(
    decompressed: &[u8],
    width: usize,
    height: usize,
    channels: usize,
) -> Option<(Vec<u8>, usize)> {
    let stride = width * channels;
    let expected = (stride + 1) * height; // +1 for filter byte per row
    if decompressed.len() < expected {
        return None;
    }

    let mut unfiltered = vec![0u8; stride * height];
    let mut prev_row = vec![0u8; stride];

    for y in 0..height {
        let row_start = y * (stride + 1);
        let filter_type = decompressed[row_start];
        let raw = &decompressed[row_start + 1..row_start + 1 + stride];

        let out_start = y * stride;
        let out = &mut unfiltered[out_start..out_start + stride];

        match filter_type {
            0 => {
                // None
                out.copy_from_slice(raw);
            }
            1 => {
                // Sub
                for i in 0..stride {
                    let a = if i >= channels { out[i - channels] } else { 0 };
                    out[i] = raw[i].wrapping_add(a);
                }
            }
            2 => {
                // Up
                for i in 0..stride {
                    out[i] = raw[i].wrapping_add(prev_row[i]);
                }
            }
            3 => {
                // Average
                for i in 0..stride {
                    let a = if i >= channels {
                        out[i - channels] as u16
                    } else {
                        0
                    };
                    let b = prev_row[i] as u16;
                    out[i] = raw[i].wrapping_add(((a + b) / 2) as u8);
                }
            }
            4 => {
                // Paeth
                for i in 0..stride {
                    let a = if i >= channels {
                        out[i - channels] as i32
                    } else {
                        0
                    };
                    let b = prev_row[i] as i32;
                    let c = if i >= channels {
                        prev_row[i - channels] as i32
                    } else {
                        0
                    };
                    out[i] = raw[i].wrapping_add(paeth_predictor(a, b, c));
                }
            }
            _ => {
                out.copy_from_slice(raw);
            }
        }

        prev_row.copy_from_slice(out);
    }

    Some((unfiltered, expected))
}

/// The longest side, in pixels, an EMF picture is rasterised at.
const EMF_RASTER_TARGET: f64 = 1024.0;
/// The largest factor an EMF's reference-device size is scaled up by.
const EMF_RASTER_MAX_SCALE: f64 = 8.0;

/// Rasterises an EMF picture ([MS-EMF]) through the records that line art,
/// logos and simple diagrams use: the window, viewport, world transform and
/// map mode, saved states, pens and solid brushes (stock objects included),
/// rectangles, rounded rectangles, ellipses, poly lines, polygons and Bezier
/// curves in their 16- and 32-bit forms, path brackets, and 1, 4, 8, 24 and
/// 32-bit `EMR_STRETCHDIBITS` bitmaps. Text, clipping, hatches, dashes and
/// EMF+ comments are not drawn. Returns `None` when no record of the subset
/// draws anything.
fn decode_emf(data: &[u8]) -> Option<DecodedImage> {
    let info = probe(data)?;
    let frame = [
        emf_i32(data, 24)?,
        emf_i32(data, 28)?,
        emf_i32(data, 32)?,
        emf_i32(data, 36)?,
    ];
    let device = (emf_i32(data, 72)?, emf_i32(data, 76)?);
    let millimetres = (emf_i32(data, 80)?, emf_i32(data, 84)?);
    // The frame in reference-device pixels: what the picture box shows.
    let px_per_hundredth = (
        f64::from(device.0) / (f64::from(millimetres.0) * 100.0),
        f64::from(device.1) / (f64::from(millimetres.1) * 100.0),
    );
    let origin = (
        f64::from(frame[0]) * px_per_hundredth.0,
        f64::from(frame[1]) * px_per_hundredth.1,
    );
    let (width, height) = (f64::from(info.width_px), f64::from(info.height_px));
    let scale = (EMF_RASTER_TARGET / width.max(height)).clamp(1.0, EMF_RASTER_MAX_SCALE);
    let mut pixmap = tiny_skia::Pixmap::new(
        (width * scale).ceil() as u32,
        (height * scale).ceil() as u32,
    )?;
    let mut player = EmfPlayer {
        state: EmfState::default(),
        saved: Vec::new(),
        objects: HashMap::new(),
        path: None,
        closed_path: None,
        origin,
        scale,
        drew: false,
    };
    let mut offset = usize::try_from(emf_u32(data, 4)?).ok()?;
    while offset + 8 <= data.len() {
        let kind = emf_u32(data, offset)?;
        let size = usize::try_from(emf_u32(data, offset + 4)?).ok()?;
        if size < 8 || size % 4 != 0 {
            break;
        }
        let record = data.get(offset..offset.checked_add(size)?)?;
        if kind == 14 {
            break; // EMR_EOF
        }
        player.play(kind, record, &mut pixmap);
        offset += size;
    }
    if !player.drew {
        return None;
    }

    let pixel_count = pixmap.width() as usize * pixmap.height() as usize;
    let mut rgb = Vec::with_capacity(pixel_count * 3);
    let mut alpha = Vec::with_capacity(pixel_count);
    for pixel in pixmap.pixels() {
        let colour = pixel.demultiply();
        rgb.extend_from_slice(&[colour.red(), colour.green(), colour.blue()]);
        alpha.push(colour.alpha());
    }
    let opaque = alpha.iter().all(|&value| value == 255);
    Some(DecodedImage {
        data: rgb,
        alpha: (!opaque).then_some(alpha),
        width: pixmap.width(),
        height: pixmap.height(),
        color_space: "DeviceRGB",
        is_jpeg: false,
    })
}

fn emf_u32(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        data.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn emf_i32(data: &[u8], offset: usize) -> Option<i32> {
    emf_u32(data, offset).map(|value| value as i32)
}

fn emf_i16(data: &[u8], offset: usize) -> Option<i16> {
    Some(i16::from_le_bytes(
        data.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn emf_f32(data: &[u8], offset: usize) -> Option<f32> {
    emf_u32(data, offset).map(f32::from_bits)
}

/// A COLORREF is 0x00BBGGRR.
fn emf_colour(value: u32) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba8(
        (value & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        ((value >> 16) & 0xff) as u8,
        255,
    )
}

#[derive(Clone, Copy)]
enum EmfObject {
    /// A pen colour and logical width, or the null pen.
    Pen(Option<(tiny_skia::Color, f64)>),
    /// A solid brush colour, or the null brush.
    Brush(Option<tiny_skia::Color>),
    Other,
}

#[derive(Clone)]
struct EmfState {
    world: [f64; 6],
    window_origin: (f64, f64),
    window_extent: (f64, f64),
    viewport_origin: (f64, f64),
    viewport_extent: (f64, f64),
    map_mode: u32,
    pen: Option<(tiny_skia::Color, f64)>,
    brush: Option<tiny_skia::Color>,
    fill_rule: tiny_skia::FillRule,
    current: (f64, f64),
}

impl Default for EmfState {
    fn default() -> Self {
        Self {
            world: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            window_origin: (0.0, 0.0),
            window_extent: (1.0, 1.0),
            viewport_origin: (0.0, 0.0),
            viewport_extent: (1.0, 1.0),
            map_mode: 1,
            pen: Some((tiny_skia::Color::BLACK, 0.0)),
            brush: Some(tiny_skia::Color::WHITE),
            fill_rule: tiny_skia::FillRule::EvenOdd,
            current: (0.0, 0.0),
        }
    }
}

struct EmfPlayer {
    state: EmfState,
    saved: Vec<EmfState>,
    objects: HashMap<u32, EmfObject>,
    /// The figure being built inside BEGINPATH and ENDPATH.
    path: Option<tiny_skia::PathBuilder>,
    /// The path ENDPATH closed, for FILLPATH and STROKEPATH.
    closed_path: Option<tiny_skia::Path>,
    /// The picture frame's top-left corner in device pixels.
    origin: (f64, f64),
    scale: f64,
    drew: bool,
}

impl EmfPlayer {
    /// Logical coordinates to raster pixels: world transform, then the
    /// window-to-viewport mapping, then the frame.
    fn map(&self, (x, y): (f64, f64)) -> (f32, f32) {
        let [m11, m12, m21, m22, dx, dy] = self.state.world;
        let page = (m11 * x + m21 * y + dx, m12 * x + m22 * y + dy);
        let state = &self.state;
        let (sx, sy) = if matches!(state.map_mode, 7 | 8)
            && state.window_extent.0 != 0.0
            && state.window_extent.1 != 0.0
        {
            (
                state.viewport_extent.0 / state.window_extent.0,
                state.viewport_extent.1 / state.window_extent.1,
            )
        } else {
            (1.0, 1.0)
        };
        let device = (
            (page.0 - state.window_origin.0) * sx + state.viewport_origin.0,
            (page.1 - state.window_origin.1) * sy + state.viewport_origin.1,
        );
        (
            ((device.0 - self.origin.0) * self.scale) as f32,
            ((device.1 - self.origin.1) * self.scale) as f32,
        )
    }

    /// How many raster pixels one logical unit spans, averaged over x and y.
    fn length_scale(&self) -> f64 {
        let zero = self.map((0.0, 0.0));
        let x = self.map((1.0, 0.0));
        let y = self.map((0.0, 1.0));
        let distance = |a: (f32, f32), b: (f32, f32)| f64::from((a.0 - b.0).hypot(a.1 - b.1));
        (distance(zero, x) + distance(zero, y)) / 2.0
    }

    fn play(&mut self, kind: u32, record: &[u8], pixmap: &mut tiny_skia::Pixmap) {
        let _ = self.play_record(kind, record, pixmap);
    }

    fn play_record(
        &mut self,
        kind: u32,
        record: &[u8],
        pixmap: &mut tiny_skia::Pixmap,
    ) -> Option<()> {
        let pair = |offset: usize| -> Option<(f64, f64)> {
            Some((
                f64::from(emf_i32(record, offset)?),
                f64::from(emf_i32(record, offset + 4)?),
            ))
        };
        match kind {
            9 => self.state.window_extent = pair(8)?,
            10 => self.state.window_origin = pair(8)?,
            11 => self.state.viewport_extent = pair(8)?,
            12 => self.state.viewport_origin = pair(8)?,
            17 => self.state.map_mode = emf_u32(record, 8)?,
            19 => {
                self.state.fill_rule = if emf_u32(record, 8)? == 2 {
                    tiny_skia::FillRule::Winding
                } else {
                    tiny_skia::FillRule::EvenOdd
                }
            }
            33 => self.saved.push(self.state.clone()),
            34 => {
                let relative = emf_i32(record, 8)?;
                for _ in 0..relative.unsigned_abs().max(1) {
                    if let Some(state) = self.saved.pop() {
                        self.state = state;
                    }
                }
            }
            35 | 36 => {
                let mut xform = [0.0; 6];
                for (index, value) in xform.iter_mut().enumerate() {
                    *value = f64::from(emf_f32(record, 8 + index * 4)?);
                }
                let mode = if kind == 35 { 4 } else { emf_u32(record, 32)? };
                self.state.world = match mode {
                    1 => [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                    2 => emf_multiply(xform, self.state.world),
                    3 => emf_multiply(self.state.world, xform),
                    _ => xform,
                };
            }
            37 => {
                let handle = emf_u32(record, 8)?;
                let object = if handle & 0x8000_0000 != 0 {
                    emf_stock_object(handle & 0x7fff_ffff)
                } else {
                    *self.objects.get(&handle)?
                };
                match object {
                    EmfObject::Pen(pen) => self.state.pen = pen,
                    EmfObject::Brush(brush) => self.state.brush = brush,
                    EmfObject::Other => {}
                }
            }
            38 => {
                let style = emf_u32(record, 12)?;
                let pen = (style & 0xf != 5).then(|| {
                    (
                        emf_colour(emf_u32(record, 24).unwrap_or(0)),
                        f64::from(emf_i32(record, 16).unwrap_or(0)),
                    )
                });
                self.objects
                    .insert(emf_u32(record, 8)?, EmfObject::Pen(pen));
            }
            95 => {
                let style = emf_u32(record, 28)?;
                let brush_style = emf_u32(record, 36)?;
                let pen = (style & 0xf != 5 && brush_style != 1).then(|| {
                    (
                        emf_colour(emf_u32(record, 40).unwrap_or(0)),
                        f64::from(emf_u32(record, 32).unwrap_or(0)),
                    )
                });
                self.objects
                    .insert(emf_u32(record, 8)?, EmfObject::Pen(pen));
            }
            39 => {
                let brush = (emf_u32(record, 12)? != 1)
                    .then(|| emf_colour(emf_u32(record, 16).unwrap_or(0)));
                self.objects
                    .insert(emf_u32(record, 8)?, EmfObject::Brush(brush));
            }
            40 => {
                self.objects.remove(&emf_u32(record, 8)?);
            }
            // Any other object creation still takes its handle slot.
            82 | 93 | 94 => {
                self.objects.insert(emf_u32(record, 8)?, EmfObject::Other);
            }
            27 => {
                self.state.current = pair(8)?;
                let (x, y) = self.map(self.state.current);
                if let Some(path) = &mut self.path {
                    path.move_to(x, y);
                }
            }
            54 => {
                let to = pair(8)?;
                self.figure(&[self.state.current, to], false, false, pixmap);
                self.state.current = to;
            }
            42..=44 => {
                let (left, top) = pair(8)?;
                let (right, bottom) = pair(16)?;
                let corner = if kind == 44 { pair(24)? } else { (0.0, 0.0) };
                let points = emf_box_outline(kind, left, top, right, bottom, corner);
                self.bezier_figure(&points, pixmap);
            }
            2..=6 | 85..=89 => {
                let wide = kind < 80;
                let count = usize::try_from(emf_u32(record, 24)?).ok()?;
                let points = emf_points(record, 28, count, wide)?;
                let base = if wide { kind } else { kind - 83 };
                match base {
                    // POLYBEZIER, POLYBEZIERTO
                    2 | 5 => {
                        let mut all = Vec::with_capacity(points.len() + 1);
                        if base == 5 {
                            all.push(self.state.current);
                        }
                        all.extend(points);
                        if let Some(last) = all.last() {
                            self.state.current = *last;
                        }
                        self.curve(&all, pixmap);
                    }
                    // POLYGON, POLYLINE, POLYLINETO
                    _ => {
                        let mut all = Vec::with_capacity(points.len() + 1);
                        if base == 6 {
                            all.push(self.state.current);
                        }
                        all.extend(points);
                        if base != 3
                            && let Some(last) = all.last()
                        {
                            self.state.current = *last;
                        }
                        self.figure(&all, base == 3, base == 3, pixmap);
                    }
                }
            }
            7 | 8 | 90 | 91 => {
                let wide = kind < 80;
                let polygons = usize::try_from(emf_u32(record, 24)?).ok()?;
                let total = usize::try_from(emf_u32(record, 28)?).ok()?;
                let counts = (0..polygons)
                    .map(|index| usize::try_from(emf_u32(record, 32 + index * 4)?).ok())
                    .collect::<Option<Vec<_>>>()?;
                let points = emf_points(record, 32 + polygons * 4, total, wide)?;
                let closed = matches!(kind, 8 | 91);
                let mut builder = tiny_skia::PathBuilder::new();
                let mut start = 0usize;
                for count in counts {
                    let figure = points.get(start..start.checked_add(count)?)?;
                    start += count;
                    self.add_polyline(&mut builder, figure, closed);
                }
                self.finish(builder, closed, pixmap);
            }
            59 => {
                self.path = Some(tiny_skia::PathBuilder::new());
                self.closed_path = None;
            }
            60 => {
                self.closed_path = self.path.take().and_then(tiny_skia::PathBuilder::finish);
            }
            61 => {
                if let Some(path) = &mut self.path {
                    path.close();
                }
            }
            62..=64 => {
                let path = self
                    .closed_path
                    .take()
                    .or_else(|| self.path.take().and_then(tiny_skia::PathBuilder::finish))?;
                let state = self.state.clone();
                if kind != 64 {
                    self.fill(&path, state.brush, pixmap);
                }
                if kind != 62 {
                    self.stroke(&path, state.pen, pixmap);
                }
            }
            81 => self.stretch_dib(record, pixmap)?,
            _ => {}
        }
        Some(())
    }

    fn add_polyline(
        &self,
        builder: &mut tiny_skia::PathBuilder,
        points: &[(f64, f64)],
        closed: bool,
    ) {
        for (index, point) in points.iter().enumerate() {
            let (x, y) = self.map(*point);
            if index == 0 {
                builder.move_to(x, y);
            } else {
                builder.line_to(x, y);
            }
        }
        if closed && points.len() > 2 {
            builder.close();
        }
    }

    /// Draws or, inside a path bracket, adds a poly line or polygon.
    fn figure(
        &mut self,
        points: &[(f64, f64)],
        closed: bool,
        filled: bool,
        pixmap: &mut tiny_skia::Pixmap,
    ) {
        if points.len() < 2 {
            return;
        }
        if let Some(mut path) = self.path.take() {
            for (index, point) in points.iter().enumerate() {
                let (x, y) = self.map(*point);
                if index == 0 && path.last_point().is_none() {
                    path.move_to(x, y);
                } else if index > 0 {
                    path.line_to(x, y);
                }
            }
            if closed {
                path.close();
            }
            self.path = Some(path);
            return;
        }
        let mut builder = tiny_skia::PathBuilder::new();
        self.add_polyline(&mut builder, points, closed);
        self.finish(builder, filled, pixmap);
    }

    /// Draws or adds a run of cubic Bezier segments: a start point, then
    /// three points per segment.
    fn curve(&mut self, points: &[(f64, f64)], pixmap: &mut tiny_skia::Pixmap) {
        let Some(first) = points.first() else {
            return;
        };
        let in_path = self.path.is_some();
        let mut builder = self.path.take().unwrap_or_default();
        let (x, y) = self.map(*first);
        if !in_path || builder.last_point().is_none() {
            builder.move_to(x, y);
        }
        for segment in points[1..].chunks_exact(3) {
            let [a, b, c] = [segment[0], segment[1], segment[2]].map(|point| self.map(point));
            builder.cubic_to(a.0, a.1, b.0, b.1, c.0, c.1);
        }
        if in_path {
            self.path = Some(builder);
        } else {
            self.finish(builder, false, pixmap);
        }
    }

    /// A closed outline of Bezier segments (rectangle, ellipse, rounded
    /// rectangle), drawn with the brush and the pen.
    fn bezier_figure(&mut self, points: &[(f64, f64)], pixmap: &mut tiny_skia::Pixmap) {
        let in_path = self.path.is_some();
        let mut builder = self.path.take().unwrap_or_default();
        let (x, y) = self.map(points[0]);
        builder.move_to(x, y);
        for segment in points[1..].chunks_exact(3) {
            let [a, b, c] = [segment[0], segment[1], segment[2]].map(|point| self.map(point));
            builder.cubic_to(a.0, a.1, b.0, b.1, c.0, c.1);
        }
        builder.close();
        if in_path {
            self.path = Some(builder);
        } else {
            self.finish(builder, true, pixmap);
        }
    }

    fn finish(
        &mut self,
        builder: tiny_skia::PathBuilder,
        filled: bool,
        pixmap: &mut tiny_skia::Pixmap,
    ) {
        let Some(path) = builder.finish() else {
            return;
        };
        let state = self.state.clone();
        if filled {
            self.fill(&path, state.brush, pixmap);
        }
        self.stroke(&path, state.pen, pixmap);
    }

    fn fill(
        &mut self,
        path: &tiny_skia::Path,
        brush: Option<tiny_skia::Color>,
        pixmap: &mut tiny_skia::Pixmap,
    ) {
        let Some(colour) = brush else {
            return;
        };
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(colour);
        paint.anti_alias = true;
        pixmap.fill_path(
            path,
            &paint,
            self.state.fill_rule,
            tiny_skia::Transform::identity(),
            None,
        );
        self.drew = true;
    }

    fn stroke(
        &mut self,
        path: &tiny_skia::Path,
        pen: Option<(tiny_skia::Color, f64)>,
        pixmap: &mut tiny_skia::Pixmap,
    ) {
        let Some((colour, width)) = pen else {
            return;
        };
        // A zero width is one device pixel wide, whatever the transform.
        let width = if width <= 0.0 {
            self.scale
        } else {
            (width * self.length_scale()).max(1.0)
        };
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(colour);
        paint.anti_alias = true;
        let stroke = tiny_skia::Stroke {
            width: width as f32,
            ..tiny_skia::Stroke::default()
        };
        pixmap.stroke_path(
            path,
            &paint,
            &stroke,
            tiny_skia::Transform::identity(),
            None,
        );
        self.drew = true;
    }

    /// EMR_STRETCHDIBITS: a device-independent bitmap stretched into a
    /// logical rectangle.
    fn stretch_dib(&mut self, record: &[u8], pixmap: &mut tiny_skia::Pixmap) -> Option<()> {
        let destination = (
            f64::from(emf_i32(record, 24)?),
            f64::from(emf_i32(record, 28)?),
        );
        let size = (
            f64::from(emf_i32(record, 72)?),
            f64::from(emf_i32(record, 76)?),
        );
        let info_offset = usize::try_from(emf_u32(record, 48)?).ok()?;
        let bits_offset = usize::try_from(emf_u32(record, 56)?).ok()?;
        let bits_size = usize::try_from(emf_u32(record, 60)?).ok()?;
        let info = record.get(info_offset..)?;
        let bits = record.get(bits_offset..bits_offset.checked_add(bits_size)?)?;
        let bitmap = emf_dib_pixmap(info, bits)?;
        let a = self.map(destination);
        let b = self.map((destination.0 + size.0, destination.1 + size.1));
        let (left, top) = (a.0.min(b.0), a.1.min(b.1));
        let (width, height) = ((a.0 - b.0).abs(), (a.1 - b.1).abs());
        if width <= 0.0 || height <= 0.0 {
            return None;
        }
        let transform = tiny_skia::Transform::from_row(
            width / bitmap.width() as f32,
            0.0,
            0.0,
            height / bitmap.height() as f32,
            left,
            top,
        );
        let paint = tiny_skia::PixmapPaint {
            quality: tiny_skia::FilterQuality::Bilinear,
            ..tiny_skia::PixmapPaint::default()
        };
        pixmap.draw_pixmap(0, 0, bitmap.as_ref(), &paint, transform, None);
        self.drew = true;
        Some(())
    }
}

/// `a` then `b`, as GDI XFORMs (eM11, eM12, eM21, eM22, eDx, eDy).
fn emf_multiply(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

fn emf_stock_object(index: u32) -> EmfObject {
    let grey = |level: u8| tiny_skia::Color::from_rgba8(level, level, level, 255);
    match index {
        0 => EmfObject::Brush(Some(tiny_skia::Color::WHITE)),
        1 => EmfObject::Brush(Some(grey(0xc0))),
        2 => EmfObject::Brush(Some(grey(0x80))),
        3 => EmfObject::Brush(Some(grey(0x40))),
        4 => EmfObject::Brush(Some(tiny_skia::Color::BLACK)),
        5 => EmfObject::Brush(None),
        6 => EmfObject::Pen(Some((tiny_skia::Color::WHITE, 0.0))),
        7 => EmfObject::Pen(Some((tiny_skia::Color::BLACK, 0.0))),
        8 => EmfObject::Pen(None),
        _ => EmfObject::Other,
    }
}

fn emf_points(record: &[u8], offset: usize, count: usize, wide: bool) -> Option<Vec<(f64, f64)>> {
    let step = if wide { 8 } else { 4 };
    record.get(offset..offset.checked_add(count.checked_mul(step)?)?)?;
    (0..count)
        .map(|index| {
            let at = offset + index * step;
            Some(if wide {
                (
                    f64::from(emf_i32(record, at)?),
                    f64::from(emf_i32(record, at + 4)?),
                )
            } else {
                (
                    f64::from(emf_i16(record, at)?),
                    f64::from(emf_i16(record, at + 2)?),
                )
            })
        })
        .collect()
}

/// The outline of EMR_ELLIPSE (42), EMR_RECTANGLE (43) or EMR_ROUNDRECT (44)
/// as a start point and cubic Bezier segments, in logical coordinates, so a
/// rotating world transform keeps its shape.
fn emf_box_outline(
    kind: u32,
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    corner: (f64, f64),
) -> Vec<(f64, f64)> {
    const KAPPA: f64 = 0.552_284_749_830_793_4;
    let line = |from: (f64, f64), to: (f64, f64)| {
        [
            (
                from.0 + (to.0 - from.0) / 3.0,
                from.1 + (to.1 - from.1) / 3.0,
            ),
            (
                from.0 + (to.0 - from.0) * 2.0 / 3.0,
                from.1 + (to.1 - from.1) * 2.0 / 3.0,
            ),
            to,
        ]
    };
    let (rx, ry) = match kind {
        42 => ((right - left) / 2.0, (bottom - top) / 2.0),
        44 => (
            (corner.0 / 2.0).min((right - left) / 2.0),
            (corner.1 / 2.0).min((bottom - top) / 2.0),
        ),
        _ => (0.0, 0.0),
    };
    let (kx, ky) = (rx * KAPPA, ry * KAPPA);
    let mut points = vec![(left + rx, top)];
    let push_line = |points: &mut Vec<(f64, f64)>, to: (f64, f64)| {
        let from = *points.last().expect("outline has a start");
        points.extend(line(from, to));
    };
    push_line(&mut points, (right - rx, top));
    points.extend([
        (right - rx + kx, top),
        (right, top + ry - ky),
        (right, top + ry),
    ]);
    push_line(&mut points, (right, bottom - ry));
    points.extend([
        (right, bottom - ry + ky),
        (right - rx + kx, bottom),
        (right - rx, bottom),
    ]);
    push_line(&mut points, (left + rx, bottom));
    points.extend([
        (left + rx - kx, bottom),
        (left, bottom - ry + ky),
        (left, bottom - ry),
    ]);
    push_line(&mut points, (left, top + ry));
    points.extend([
        (left, top + ry - ky),
        (left + rx - kx, top),
        (left + rx, top),
    ]);
    points
}

/// Decodes an uncompressed (BI_RGB, or BI_BITFIELDS with the usual masks)
/// 1, 4, 8, 24 or 32-bit device-independent bitmap.
fn emf_dib_pixmap(info: &[u8], bits: &[u8]) -> Option<tiny_skia::Pixmap> {
    let header_size = usize::try_from(emf_u32(info, 0)?).ok()?;
    let width = emf_i32(info, 4)?;
    let height = emf_i32(info, 8)?;
    let bit_count = u16::from_le_bytes(info.get(14..16)?.try_into().ok()?);
    let compression = emf_u32(info, 16)?;
    let colours_used = emf_u32(info, 32).unwrap_or(0);
    if width <= 0 || height == 0 || !matches!(compression, 0 | 3) {
        return None;
    }
    let (width, rows) = (width as usize, height.unsigned_abs() as usize);
    let palette_count = match bit_count {
        1 | 4 | 8 if colours_used == 0 => 1usize << bit_count,
        1 | 4 | 8 => colours_used as usize,
        24 | 32 => 0,
        _ => return None,
    };
    let palette = info.get(header_size..header_size + palette_count * 4)?;
    let stride = (width * usize::from(bit_count)).div_ceil(32) * 4;
    bits.get(..stride.checked_mul(rows)?)?;
    let mut rgba = Vec::with_capacity(width * rows * 4);
    let mut any_alpha = false;
    for row in 0..rows {
        // A positive height stores the bottom row first.
        let source_row = if height > 0 { rows - 1 - row } else { row };
        let line = &bits[source_row * stride..(source_row + 1) * stride];
        for column in 0..width {
            let (b, g, r, a) = match bit_count {
                24 => (
                    line[column * 3],
                    line[column * 3 + 1],
                    line[column * 3 + 2],
                    255,
                ),
                32 => {
                    let alpha = line[column * 4 + 3];
                    any_alpha |= alpha != 0;
                    (
                        line[column * 4],
                        line[column * 4 + 1],
                        line[column * 4 + 2],
                        alpha,
                    )
                }
                _ => {
                    let bit = column * usize::from(bit_count);
                    let byte = line[bit / 8];
                    let shift = 8 - usize::from(bit_count) - bit % 8;
                    let index = usize::from(byte >> shift) & ((1 << bit_count) - 1);
                    let entry = palette.get(index * 4..index * 4 + 3)?;
                    (entry[0], entry[1], entry[2], 255)
                }
            };
            rgba.extend_from_slice(&[r, g, b, a]);
        }
    }
    // Most 32-bit DIBs in metafiles leave the fourth byte zero for opaque.
    for pixel in rgba.chunks_exact_mut(4) {
        if bit_count == 32 && !any_alpha {
            pixel[3] = 255;
        }
        let colour =
            tiny_skia::ColorU8::from_rgba(pixel[0], pixel[1], pixel[2], pixel[3]).premultiply();
        pixel.copy_from_slice(&[colour.red(), colour.green(), colour.blue(), colour.alpha()]);
    }
    tiny_skia::Pixmap::from_vec(
        rgba,
        tiny_skia::IntSize::from_wh(width as u32, rows as u32)?,
    )
}

fn paeth_predictor(a: i32, b: i32, c: i32) -> u8 {
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_jpeg_pass_through() {
        // Minimal JPEG: SOI + SOF0 header + EOI
        // SOI marker
        let mut jpeg = vec![0xFF, 0xD8];
        // APP0 marker (dummy)
        jpeg.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x02]);
        // SOF0 marker: length=17, precision=8, height=2, width=3, components=3
        jpeg.extend_from_slice(&[
            0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x02, 0x00, 0x03, 0x03, 0x01, 0x11, 0x00, 0x02,
            0x11, 0x00, 0x03, 0x11, 0x00,
        ]);
        // EOI
        jpeg.extend_from_slice(&[0xFF, 0xD9]);

        let result = decode_image(&jpeg, "image/jpeg");
        assert!(result.is_some());
        let decoded = result.unwrap();
        assert!(decoded.is_jpeg);
        assert_eq!(decoded.width, 3);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.color_space, "DeviceRGB");
        assert!(decoded.alpha.is_none());
    }

    #[test]
    fn decode_invalid_data_returns_none() {
        let result = decode_image(b"not an image", "image/png");
        assert!(result.is_none());
    }

    /// Build a PNG chunk. The decoder does not verify CRCs, so a zero CRC is
    /// enough to keep these fixtures readable.
    fn png_chunk(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut c = Vec::new();
        c.extend_from_slice(&(body.len() as u32).to_be_bytes());
        c.extend_from_slice(kind);
        c.extend_from_slice(body);
        c.extend_from_slice(&[0, 0, 0, 0]);
        c
    }

    /// Build an 8-bit indexed PNG from a palette and one index per pixel.
    fn indexed_png(
        width: u32,
        height: u32,
        palette: &[u8],
        trns: Option<&[u8]>,
        indices: &[u8],
    ) -> Vec<u8> {
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&width.to_be_bytes());
        ihdr.extend_from_slice(&height.to_be_bytes());
        ihdr.push(8); // bit depth
        ihdr.push(3); // colour type: indexed
        ihdr.extend_from_slice(&[0, 0, 0]); // compression, filter, interlace

        // One filter byte per scanline, filter 0 meaning no filtering.
        let mut raw = Vec::new();
        for y in 0..height as usize {
            raw.push(0);
            raw.extend_from_slice(&indices[y * width as usize..(y + 1) * width as usize]);
        }
        let idat = miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6);

        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        out.extend_from_slice(&png_chunk(b"IHDR", &ihdr));
        if !palette.is_empty() {
            out.extend_from_slice(&png_chunk(b"PLTE", palette));
        }
        if let Some(t) = trns {
            out.extend_from_slice(&png_chunk(b"tRNS", t));
        }
        out.extend_from_slice(&png_chunk(b"IDAT", &idat));
        out.extend_from_slice(&png_chunk(b"IEND", &[]));
        out
    }

    /// The EMF the issue builds: a header, a solid red brush, one filled
    /// rectangle and the end record, at 96 dpi on a 1024 x 768 px, 270 x 203
    /// mm reference device.
    fn red_rectangle_emf(width: i32, height: i32) -> Vec<u8> {
        let mut records = Vec::new();
        for words in [
            vec![39, 24, 1, 0, 0x001E_1EC8, 0], // CREATEBRUSHINDIRECT, solid red
            vec![37, 12, 1],                    // SELECTOBJECT
            vec![43, 24, 10, 10, width - 10, height - 10], // RECTANGLE
            vec![14, 20, 0, 16, 20],            // EOF
        ] {
            for word in words {
                records.extend_from_slice(&(word as u32).to_le_bytes());
            }
        }
        let mut header = Vec::new();
        for word in [1, 88, 0, 0, width - 1, height - 1, 0, 0] {
            header.extend_from_slice(&(word as u32).to_le_bytes());
        }
        for word in [
            width * 2645 / 100,
            height * 2645 / 100,
            0x464D_4520,
            0x10000,
        ] {
            header.extend_from_slice(&(word as u32).to_le_bytes());
        }
        header.extend_from_slice(&((88 + records.len()) as u32).to_le_bytes());
        header.extend_from_slice(&5u32.to_le_bytes());
        header.extend_from_slice(&2u16.to_le_bytes());
        header.extend_from_slice(&0u16.to_le_bytes());
        for word in [0, 0, 0, 1024, 768, 270, 203] {
            header.extend_from_slice(&(word as u32).to_le_bytes());
        }
        header.extend(records);
        header
    }

    /// EMF pictures were not decoded at all, so rdocx left their frame
    /// empty and rpptx left them out.
    #[test]
    fn emf_rectangle_is_rasterised_with_its_brush() {
        let decoded = decode_image(&red_rectangle_emf(200, 100), "image/x-emf")
            .expect("the EMF subset draws a filled rectangle");
        let (width, height) = (decoded.width as usize, decoded.height as usize);
        // 200 x 100 px at the reference device's 3.79 px/mm, scaled to 1024.
        assert_eq!(width, 1024);
        assert!((505..=515).contains(&height), "{height}");
        let alpha = decoded
            .alpha
            .expect("the margin around the rectangle is clear");
        let pixel = |x: usize, y: usize| {
            let at = y * width + x;
            (&decoded.data[at * 3..at * 3 + 3], alpha[at])
        };
        // The rectangle spans (10, 10) to (190, 90) of 200 x 100 device px.
        assert_eq!(pixel(width / 2, height / 2), (&[0xC8, 0x1E, 0x1E][..], 255));
        assert_eq!(
            pixel(width / 10, height / 4),
            (&[0xC8, 0x1E, 0x1E][..], 255)
        );
        assert_eq!(pixel(width / 50, height / 2).1, 0);
        assert_eq!(pixel(width / 2, height / 50).1, 0);
        // The default pen outlines the rectangle in black.
        let edge = pixel((10.0 * 1024.0 / 201.0) as usize, height / 2);
        assert!(edge.0.iter().all(|&channel| channel < 0x40), "{edge:?}");
    }

    #[test]
    fn emf_without_drawing_records_is_not_decoded() {
        let mut emf = red_rectangle_emf(200, 100);
        // Turn the rectangle into an unknown record of the same size.
        let rectangle = 88 + 24 + 12;
        emf[rectangle..rectangle + 4].copy_from_slice(&999u32.to_le_bytes());
        assert!(decode_image(&emf, "image/x-emf").is_none());
    }

    /// An interlaced PNG was read as if its seven passes were the rows of a
    /// plain image, so it was drawn as stripes of wrong colours.
    #[test]
    fn interlaced_png_decodes_like_its_plain_copy() {
        let (width, height) = (11usize, 9usize);
        let pixel = |x: usize, y: usize| [(x * 20) as u8, (y * 25) as u8, 7];
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&(width as u32).to_be_bytes());
        ihdr.extend_from_slice(&(height as u32).to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 1]); // RGB, Adam7
        // Each pass is its own sub-image, its rows filtered with Sub so the
        // filter state is reset per pass.
        let mut raw = Vec::new();
        for (x0, y0, dx, dy) in ADAM7_PASSES {
            for y in (y0..height).step_by(dy) {
                let row: Vec<u8> = (x0..width).step_by(dx).flat_map(|x| pixel(x, y)).collect();
                if row.is_empty() {
                    continue;
                }
                raw.push(1);
                raw.extend(
                    (0..row.len())
                        .map(|i| row[i].wrapping_sub(if i >= 3 { row[i - 3] } else { 0 })),
                );
            }
        }
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        png.extend_from_slice(&png_chunk(b"IHDR", &ihdr));
        png.extend_from_slice(&png_chunk(
            b"IDAT",
            &miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6),
        ));
        png.extend_from_slice(&png_chunk(b"IEND", &[]));

        let decoded = decode_image(&png, "image/png").expect("interlaced PNG decodes");
        let expected: Vec<u8> = (0..height)
            .flat_map(|y| (0..width).flat_map(move |x| pixel(x, y)))
            .collect();
        assert_eq!(
            (decoded.width, decoded.height),
            (width as u32, height as u32)
        );
        assert_eq!(decoded.data, expected);
    }

    /// Indexed PNGs used to fall into the catch-all and be dropped, so any
    /// document containing one lost that image with no warning.
    #[test]
    fn indexed_png_expands_palette_to_rgb() {
        // red, green, blue
        let palette = [255, 0, 0, 0, 255, 0, 0, 0, 255];
        let png = indexed_png(2, 2, &palette, None, &[0, 1, 2, 0]);

        let decoded = decode_image(&png, "image/png").expect("indexed PNG must decode");
        assert_eq!((decoded.width, decoded.height), (2, 2));
        assert_eq!(decoded.color_space, "DeviceRGB");
        assert!(!decoded.is_jpeg);
        assert_eq!(
            decoded.data,
            vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 0]
        );
        assert!(
            decoded.alpha.is_none(),
            "a fully opaque image should not carry an alpha channel"
        );
    }

    /// tRNS on an indexed image is per palette entry, and it may cover only
    /// the first few entries.
    #[test]
    fn indexed_png_honours_trns_alpha() {
        let palette = [255, 0, 0, 0, 255, 0];
        // Entry 0 fully transparent. Entry 1 is not mentioned, so opaque.
        let png = indexed_png(2, 1, &palette, Some(&[0]), &[0, 1]);

        let decoded = decode_image(&png, "image/png").expect("indexed PNG must decode");
        assert_eq!(decoded.data, vec![255, 0, 0, 0, 255, 0]);
        assert_eq!(decoded.alpha, Some(vec![0, 255]));
    }

    /// Colour type 3 without a PLTE chunk is malformed. Returning None is
    /// right, but it must not be confused with the palette case above.
    #[test]
    fn indexed_png_without_palette_is_rejected() {
        let png = indexed_png(1, 1, &[], None, &[0]);
        assert!(decode_image(&png, "image/png").is_none());
    }

    /// An index past the end of the palette is malformed input. It should not
    /// panic and should not discard the rest of the image.
    #[test]
    fn indexed_png_out_of_range_index_falls_back_to_black() {
        let palette = [255, 0, 0]; // a single entry, so index 5 is invalid
        let png = indexed_png(2, 1, &palette, None, &[0, 5]);

        let decoded = decode_image(&png, "image/png").expect("should still decode");
        assert_eq!(decoded.data, vec![255, 0, 0, 0, 0, 0]);
    }
}
