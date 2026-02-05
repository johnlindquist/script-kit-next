use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::ExtendedColorType;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;

#[derive(Debug, Clone)]
pub struct CameraFrame {
    pub width: u32,
    pub height: u32,
    pub frame_number: u64,
    pub data: Vec<u8>,
}

pub struct CameraCapture {
    width: u32,
    height: u32,
    frame_number: u64,
    camera: Camera,
}

impl CameraCapture {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let index = CameraIndex::Index(0);
        let mut camera =
            Camera::new(index, requested).context("Failed to initialize webcam capture")?;
        camera
            .open_stream()
            .context("Failed to open webcam stream")?;

        Ok(Self {
            width,
            height,
            frame_number: 0,
            camera,
        })
    }

    pub fn next_frame(&mut self) -> CameraFrame {
        self.frame_number = self.frame_number.saturating_add(1);

        if let Ok(frame) = self.camera.frame() {
            let resolution = frame.resolution();
            let width = resolution.width();
            let height = resolution.height();
            if let Ok(decoded) = frame.decode_image::<RgbFormat>() {
                return CameraFrame {
                    width,
                    height,
                    frame_number: self.frame_number,
                    data: decoded.into_raw(),
                };
            }
        }

        CameraFrame {
            width: self.width,
            height: self.height,
            frame_number: self.frame_number,
            data: Vec::new(),
        }
    }
}

pub fn frame_to_base64_jpeg(frame: &CameraFrame) -> Result<String> {
    if frame.data.is_empty() {
        return Ok(String::new());
    }

    let expected = (frame.width * frame.height * 3) as usize;
    if frame.data.len() != expected {
        return Err(anyhow!(
            "Frame data mismatch: expected {} bytes, got {}",
            expected,
            frame.data.len()
        ));
    }

    let mut output = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut output, 80);
    encoder.encode(
        &frame.data,
        frame.width,
        frame.height,
        ExtendedColorType::Rgb8,
    )?;

    Ok(STANDARD.encode(output))
}
