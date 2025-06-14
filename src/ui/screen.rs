use sdl2::image::{LoadSurface, LoadTexture};
pub use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
pub use sdl2::rect::{Point, Rect};
use sdl2::render::TextureQuery;
use sdl2::surface::Surface;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

#[derive(Copy, Clone)]
pub enum FontSize {
    Body,
    Title,
}

pub const SHADOW_DELTA: u32 = 2;

pub struct Screen<'a, 'b> {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    width: u32,
    height: u32,
    body_font: sdl2::ttf::Font<'a, 'b>,
    title_font: sdl2::ttf::Font<'a, 'b>,
}

impl<'a, 'b> Screen<'a, 'b> {
    pub(super) fn new(
        canvas: sdl2::render::Canvas<sdl2::video::Window>,
        width: u32,
        height: u32,
        ttf_context: &'a sdl2::ttf::Sdl2TtfContext,
        font_path: &Path,
    ) -> Self {
        let body_font = ttf_context.load_font(font_path, 14).unwrap();
        let title_font = ttf_context.load_font(font_path, 18).unwrap();

        Self {
            canvas,
            width,
            height,
            body_font,
            title_font,
        }
    }

    pub(super) fn clear(&mut self, color: Color) {
        let canvas = &mut self.canvas;
        canvas.set_draw_color(color);
        canvas.clear();
    }

    pub(super) fn present(&mut self) {
        let canvas = &mut self.canvas;
        canvas.present();
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn draw_image(&mut self, path: &Path, source: Option<Rect>, target: Option<Rect>) {
        let canvas = &mut self.canvas;
        let texture_creator = canvas.texture_creator();
        let image = texture_creator.load_texture(path).unwrap();
        canvas.copy(&image, source, target).unwrap();
    }

    pub fn draw_background(&mut self, image: sdl2::surface::Surface, scale: f32, angle: f64) {
        let canvas = &mut self.canvas;
        let screen_width = self.width;
        let screen_height = self.height;
        let (width, height) = image.size();
        let texture_creator = canvas.texture_creator();
        let image = image.as_texture(&texture_creator).unwrap();

        let h = (screen_width as f32 * height as f32 / width as f32) as u32;
        let dh = screen_height as i32 - h as i32;

        canvas
            .copy_ex(
                &image,
                None,
                Some(Rect::new(
                    ((screen_width as f32) * (scale - 1.0) * -0.5) as i32,
                    (dh as f32 * scale / 2.0) as i32
                        + ((screen_height as f32) * (scale - 1.0) * -0.5) as i32,
                    ((screen_width as f32) * scale) as u32,
                    ((h as f32) * scale) as u32,
                )),
                angle,
                Point::new(
                    (screen_width as f32 * scale / 2.0) as i32,
                    (h as f32 * scale / 2.0) as i32,
                ),
                false,
                false,
            )
            .unwrap();
    }

    pub fn draw_rect(&mut self, color: Color, rect: Rect) {
        let canvas = &mut self.canvas;
        canvas.set_draw_color(color);
        canvas.fill_rect(rect).unwrap()
    }

    pub fn measure_text(&mut self, size: FontSize, text: &str) -> (u32, u32) {
        let font = match size {
            FontSize::Body => &self.body_font,
            FontSize::Title => &self.title_font,
        };

        let canvas = &mut self.canvas;
        let texture_creator = canvas.texture_creator();
        let surface = font.render(text).blended(Color::RGB(0, 0, 0)).unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let TextureQuery { width, height, .. } = texture.query();
        (width, height)
    }

    fn draw_text_source(
        &mut self,
        size: FontSize,
        text: &str,
        x: i32,
        y: i32,
        max_width: Option<u32>,
    ) {
        let font = match size {
            FontSize::Body => &self.body_font,
            FontSize::Title => &self.title_font,
        };

        let canvas = &mut self.canvas;
        let texture_creator = canvas.texture_creator();
        for (color, delta) in [
            (Color::RGBA(0, 0, 0, 255), SHADOW_DELTA),
            (Color::RGBA(255, 255, 255, 255), 0),
        ] {
            let surface = font.render(text).blended(color).unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .unwrap();
            let TextureQuery { width, height, .. } = texture.query();
            let width = match max_width {
                Some(w) if w < width => w,
                _ => width,
            };
            let target = Rect::new(delta as i32 + x, delta as i32 + y, width, height);
            canvas
                .copy(
                    &texture,
                    max_width.map(|w| Rect::new(0, 0, w, height)),
                    Some(target),
                )
                .unwrap();
        }
    }

    pub fn draw_text(&mut self, size: FontSize, text: &str, x: i32, y: i32) {
        self.draw_text_source(size, text, x, y, None);
    }

    pub fn draw_text_clipped(
        &mut self,
        size: FontSize,
        text: &str,
        x: i32,
        y: i32,
        max_width: u32,
    ) {
        self.draw_text_source(size, text, x, y, Some(max_width))
    }

    pub fn create_background(path: &Path, desaturate: u8, darken: u8) -> sdl2::surface::Surface {
        let mut surface = Surface::from_file(path).unwrap();
        let (width, height) = surface.size();
        let pitch = surface.pitch();
        if surface.pixel_format_enum() != PixelFormatEnum::RGB24 {
            panic!("Unexpected pixel format {:?}", surface.pixel_format_enum());
        }

        surface.with_lock_mut(|bytes: &mut [u8]| {
            // desaturate
            let amount = desaturate as f32 / 255.0;
            for y in 0..height {
                let yoffset = (y * pitch) as usize;
                for x in 0..width {
                    let offset = yoffset + (x * 3) as usize;
                    let r = bytes[offset] as f32 / 255.0;
                    let g = bytes[offset + 1] as f32 / 255.0;
                    let b = bytes[offset + 2] as f32 / 255.0;
                    let z = 0.2126 * r + 0.7152 * g + 0.0722 * b;

                    bytes[offset] = ((z * amount + r * (1.0 - amount)) * 255.0) as u8;
                    bytes[offset + 1] = ((z * amount + g * (1.0 - amount)) * 255.0) as u8;
                    bytes[offset + 2] = ((z * amount + b * (1.0 - amount)) * 255.0) as u8;
                }
            }

            // darken
            let amount = 1.0 - (darken as f32 / 255.0);
            for b in bytes {
                *b = (*b as f32 * amount) as u8;
            }
        });

        surface
    }

    pub fn recommended_margin(&self) -> u32 {
        ((self.width as f32) / 40.0) as u32
    }

    pub fn blur_rect(&mut self, rect: Rect, radius: u32) {
        if radius == 0 {
            return;
        }

        let format = sdl2::pixels::PixelFormatEnum::ARGB8888;

        // Read original pixels for blending
        let original_pixels = match self.canvas.read_pixels(Some(rect), format) {
            Ok(pixels) => pixels,
            Err(_) => return,
        };

        // Expand the read area to include blur radius padding
        let padding = radius;
        let read_x = (rect.x() - padding as i32).max(0);
        let read_y = (rect.y() - padding as i32).max(0);
        let read_width = (rect.width() + padding * 2).min(self.width - read_x as u32);
        let read_height = (rect.height() + padding * 2).min(self.height - read_y as u32);

        let read_rect = Rect::new(read_x, read_y, read_width, read_height);

        // Read pixels from the expanded area
        let pixels = match self.canvas.read_pixels(Some(read_rect), format) {
            Ok(pixels) => pixels,
            Err(_) => return,
        };

        // Apply basic gaussian blur
        let blurred_pixels =
            self.apply_gaussian_blur(&pixels, read_rect.width(), read_rect.height(), radius);

        // Calculate the offset of the original rect within the read area
        let offset_x = rect.x() - read_rect.x();
        let offset_y = rect.y() - read_rect.y();

        // Extract and feather the original rect area from the blurred result
        let mut final_pixels = Vec::with_capacity((rect.width() * rect.height() * 4) as usize);
        let read_width = read_rect.width() as usize;
        let feather_distance = radius as f32;

        for y in 0..rect.height() {
            for x in 0..rect.width() {
                let src_x = (x as i32 + offset_x) as usize;
                let src_y = (y as i32 + offset_y) as usize;
                let src_idx = (src_y * read_width + src_x) * 4;
                let orig_idx = (y * rect.width() + x) as usize * 4;

                // Calculate distance to nearest edge for feathering, excluding screen edges
                let mut min_edge_dist = feather_distance; // Start with max distance

                // Check left edge (only if not at screen edge)
                if rect.x() > 0 {
                    min_edge_dist = min_edge_dist.min(x as f32);
                }

                // Check right edge (only if not at screen edge)
                if rect.x() + (rect.width() as i32) < (self.width as i32) {
                    min_edge_dist = min_edge_dist.min((rect.width() - 1 - x) as f32);
                }

                // Check top edge (only if not at screen edge)
                if rect.y() > 0 {
                    min_edge_dist = min_edge_dist.min(y as f32);
                }

                // Check bottom edge (only if not at screen edge)
                if rect.y() + (rect.height() as i32) < (self.height as i32) {
                    min_edge_dist = min_edge_dist.min((rect.height() - 1 - y) as f32);
                }

                // Calculate feather factor (0.0 = original, 1.0 = fully blurred)
                let feather_factor = if min_edge_dist >= feather_distance {
                    1.0
                } else {
                    min_edge_dist / feather_distance
                };

                // Blend original and blurred pixels based on feather factor
                let blur_r = blurred_pixels[src_idx] as f32;
                let blur_g = blurred_pixels[src_idx + 1] as f32;
                let blur_b = blurred_pixels[src_idx + 2] as f32;
                let blur_a = blurred_pixels[src_idx + 3] as f32;

                let orig_r = original_pixels[orig_idx] as f32;
                let orig_g = original_pixels[orig_idx + 1] as f32;
                let orig_b = original_pixels[orig_idx + 2] as f32;
                let orig_a = original_pixels[orig_idx + 3] as f32;

                let final_r = orig_r + (blur_r - orig_r) * feather_factor;
                let final_g = orig_g + (blur_g - orig_g) * feather_factor;
                let final_b = orig_b + (blur_b - orig_b) * feather_factor;
                let final_a = orig_a + (blur_a - orig_a) * feather_factor;

                final_pixels.push(final_r.clamp(0.0, 255.0) as u8);
                final_pixels.push(final_g.clamp(0.0, 255.0) as u8);
                final_pixels.push(final_b.clamp(0.0, 255.0) as u8);
                final_pixels.push(final_a.clamp(0.0, 255.0) as u8);
            }
        }

        // Create surface and draw back to original rect
        if let Ok(surface) = sdl2::surface::Surface::from_data(
            &mut final_pixels,
            rect.width(),
            rect.height(),
            format.byte_size_of_pixels(rect.width() as usize) as u32,
            format,
        ) {
            let texture_creator = self.canvas.texture_creator();
            if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
                let _ = self.canvas.copy(&texture, None, Some(rect));
            }
        }
    }

    fn apply_gaussian_blur(&self, pixels: &[u8], width: u32, height: u32, radius: u32) -> Vec<u8> {
        if radius == 0 {
            return pixels.to_vec();
        }

        let w = width as usize;
        let h = height as usize;
        let r = radius as usize;

        // Generate gaussian kernel
        let sigma = radius as f32 / 3.0; // Standard sigma relationship
        let kernel_size = (radius * 2 + 1) as usize;
        let mut kernel = vec![0.0f32; kernel_size];
        let mut kernel_sum = 0.0f32;

        for (i, kernel_item) in kernel.iter_mut().enumerate().take(kernel_size) {
            let x = (i as i32 - r as i32) as f32;
            let value = (-x * x / (2.0 * sigma * sigma)).exp();
            *kernel_item = value;
            kernel_sum += value;
        }

        // Normalize kernel
        for weight in &mut kernel {
            *weight /= kernel_sum;
        }

        let mut temp_result = vec![0u8; pixels.len()];

        // Horizontal pass
        for y in 0..h {
            for x in 0..w {
                let mut r_sum = 0.0f32;
                let mut g_sum = 0.0f32;
                let mut b_sum = 0.0f32;

                for (i, &weight) in kernel.iter().enumerate() {
                    let dx = x as i32 + i as i32 - r as i32;
                    if dx >= 0 && (dx as usize) < w {
                        let idx = (y * w + dx as usize) * 4;
                        b_sum += pixels[idx] as f32 * weight;
                        g_sum += pixels[idx + 1] as f32 * weight;
                        r_sum += pixels[idx + 2] as f32 * weight;
                    }
                }

                let idx = (y * w + x) * 4;
                temp_result[idx] = b_sum.clamp(0.0, 255.0) as u8;
                temp_result[idx + 1] = g_sum.clamp(0.0, 255.0) as u8;
                temp_result[idx + 2] = r_sum.clamp(0.0, 255.0) as u8;
                temp_result[idx + 3] = pixels[idx + 3];
            }
        }

        let mut final_result = vec![0u8; pixels.len()];

        // Vertical pass
        for y in 0..h {
            for x in 0..w {
                let mut r_sum = 0.0f32;
                let mut g_sum = 0.0f32;
                let mut b_sum = 0.0f32;

                for (i, &weight) in kernel.iter().enumerate() {
                    let dy = y as i32 + i as i32 - r as i32;
                    if dy >= 0 && (dy as usize) < h {
                        let idx = (dy as usize * w + x) * 4;
                        b_sum += temp_result[idx] as f32 * weight;
                        g_sum += temp_result[idx + 1] as f32 * weight;
                        r_sum += temp_result[idx + 2] as f32 * weight;
                    }
                }

                let idx = (y * w + x) * 4;
                final_result[idx] = b_sum.clamp(0.0, 255.0) as u8;
                final_result[idx + 1] = g_sum.clamp(0.0, 255.0) as u8;
                final_result[idx + 2] = r_sum.clamp(0.0, 255.0) as u8;
                final_result[idx + 3] = temp_result[idx + 3];
            }
        }

        final_result
    }

    pub fn take_screenshot(&self) {
        let filename = format!(
            "{}.bmp",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let format = sdl2::pixels::PixelFormatEnum::ARGB8888;
        let (width, height) = self.size();
        let pitch = format.byte_size_of_pixels(width as usize) as u32;

        match self.canvas.read_pixels(None, format).and_then(|mut bytes| {
            let surface = sdl2::surface::Surface::from_data(
                bytes.as_mut_slice(),
                width,
                height,
                pitch,
                format,
            )
            .unwrap();
            surface
                .save_bmp(&filename)
                .map_err(|e| format!("Could not write screenshot: {e:?}"))
        }) {
            Ok(_) => {
                info!("Saved screenshot {filename:?}");
            }
            Err(e) => {
                error!("Could not save screenshot: {e:?}");
            }
        };
    }
}
