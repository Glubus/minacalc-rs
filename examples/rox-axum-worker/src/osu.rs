use reqwest::{Client, Url};

use crate::{error::ApiError, models::ChartPayload};

const MAX_DOWNLOAD_BYTES: u64 = 16 * 1024 * 1024;

pub(crate) async fn download(client: &Client, input: &str) -> Result<ChartPayload, ApiError> {
    let beatmap_id = beatmap_id(input)?;
    let download_url = format!("https://osu.ppy.sh/osu/{beatmap_id}");
    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|error| ApiError::bad_gateway(format!("osu! download failed: {error}")))?;

    if !response.status().is_success() {
        return Err(ApiError::bad_gateway(format!(
            "osu! returned HTTP {} for beatmap {beatmap_id}",
            response.status()
        )));
    }
    if response
        .content_length()
        .is_some_and(|size| size > MAX_DOWNLOAD_BYTES)
    {
        return Err(ApiError::payload_too_large(
            "downloaded osu! beatmap exceeds 16 MiB",
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|error| ApiError::bad_gateway(format!("could not read osu! map: {error}")))?;
    if bytes.len() as u64 > MAX_DOWNLOAD_BYTES {
        return Err(ApiError::payload_too_large(
            "downloaded osu! beatmap exceeds 16 MiB",
        ));
    }

    Ok(ChartPayload {
        bytes: bytes.to_vec(),
        file_name: Some(format!("{beatmap_id}.osu")),
    })
}

fn beatmap_id(input: &str) -> Result<u64, ApiError> {
    let url = Url::parse(input.trim())
        .map_err(|_| ApiError::bad_request("osu_url must be a valid URL"))?;
    if url.scheme() != "https" || url.host_str() != Some("osu.ppy.sh") {
        return Err(ApiError::bad_request("osu_url must use https://osu.ppy.sh"));
    }

    direct_download_id(&url)
        .or_else(|| fragment_beatmap_id(&url))
        .ok_or_else(|| ApiError::bad_request("could not find a beatmap ID in osu_url"))
}

fn direct_download_id(url: &Url) -> Option<u64> {
    let mut segments = url.path_segments()?;
    if let (Some("osu" | "beatmaps" | "b"), Some(id)) = (segments.next(), segments.next()) {
        positive_id(id)
    } else {
        None
    }
}

fn fragment_beatmap_id(url: &Url) -> Option<u64> {
    url.fragment()?.rsplit('/').next().and_then(positive_id)
}

fn positive_id(value: &str) -> Option<u64> {
    value.parse::<u64>().ok().filter(|id| *id > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_id_from_beatmapset_and_direct_urls() {
        assert_eq!(
            beatmap_id("https://osu.ppy.sh/beatmapsets/1856758#mania/3816042").unwrap(),
            3_816_042
        );
        assert_eq!(
            beatmap_id("https://osu.ppy.sh/osu/3816042").unwrap(),
            3_816_042
        );
    }

    #[test]
    fn rejects_non_osu_hosts() {
        assert!(beatmap_id("https://example.com/osu/3816042").is_err());
        assert!(beatmap_id("https://osu.ppy.sh.evil.test/osu/3816042").is_err());
    }
}
