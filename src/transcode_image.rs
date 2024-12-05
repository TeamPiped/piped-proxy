use actix_web::{Either, HttpResponse, HttpResponseBuilder};
use image::ImageError;
use once_cell::sync::Lazy;
use reqwest::Response;
use tokio::task::spawn_blocking;

pub static DISALLOW_IMAGE_TRANSCODING: Lazy<bool> =
    Lazy::new(|| crate::utils::get_env_bool("DISALLOW_IMAGE_TRANSCODING"));

#[derive(Debug, thiserror::Error)]
enum ImageTranscodingError {
    #[error("Image loading error: {0}")]
    ImageLoadingError(#[from] ImageError),
    #[cfg(feature = "webp")]
    #[error("Image is not an 8bit RGB")]
    NotAnRgb8Image,
}

pub async fn transcode_image(
    image_response: Response,
    http_response: &mut HttpResponseBuilder,
    #[cfg(feature = "avif")] avif: bool,
) -> Result<Either<HttpResponse, Response>, Box<dyn std::error::Error>> {
    let Some(content_type) = image_response.headers().get("content-type") else {
        return Ok(Either::Right(image_response));
    };

    #[cfg(feature = "avif")]
    if content_type == "image/webp" || content_type == "image/jpeg" && avif {
        let resp_bytes = image_response.bytes().await?;

        let (body, content_type) = spawn_blocking(
            || -> Result<(Vec<u8>, &'static str), ImageTranscodingError> {
                use ravif::{Encoder, Img};
                use rgb::FromSlice;

                let image = image::load_from_memory(&resp_bytes)?;

                let width = image.width() as usize;
                let height = image.height() as usize;

                let buf = image.into_rgb8();
                let buf = buf.as_raw().as_rgb();

                let buffer = Img::new(buf, width, height);

                let res = Encoder::new()
                    .with_quality(80f32)
                    .with_speed(7)
                    .encode_rgb(buffer);

                if let Ok(res) = res {
                    Ok((res.avif_file, "image/avif"))
                } else {
                    Ok((resp_bytes.into(), "image/jpeg"))
                }
            },
        )
        .await??;

        http_response.content_type(content_type);
        return Ok(Either::Left(http_response.body(body)));
    }

    #[cfg(feature = "webp")]
    if content_type == "image/jpeg" {
        let resp_bytes = image_response.bytes().await?;

        let (body, content_type) = spawn_blocking(
            || -> Result<(Vec<u8>, &'static str), ImageTranscodingError> {
                use libwebp_sys::{WebPEncodeRGB, WebPFree};

                let image = image::load_from_memory(&resp_bytes)?;
                let width = image.width();
                let height = image.height();

                let quality = 85;

                let data = image
                    .as_rgb8()
                    .ok_or(ImageTranscodingError::NotAnRgb8Image)?
                    .as_raw();

                let bytes: Vec<u8> = unsafe {
                    let mut out_buf = std::ptr::null_mut();
                    let stride = width as i32 * 3;
                    let len: usize = WebPEncodeRGB(
                        data.as_ptr(),
                        width as i32,
                        height as i32,
                        stride,
                        quality as f32,
                        &mut out_buf,
                    );
                    let vec = std::slice::from_raw_parts(out_buf, len).into();
                    WebPFree(out_buf as *mut _);
                    vec
                };

                if bytes.len() < resp_bytes.len() {
                    Ok((bytes, "image/webp"))
                } else {
                    Ok((resp_bytes.into(), "image/jpeg"))
                }
            },
        )
        .await??;

        http_response.content_type(content_type);
        return Ok(Either::Left(http_response.body(body)));
    }

    Ok(Either::Right(image_response))
}
