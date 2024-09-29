use crate::extractors::common::{Chroot, ExtractionResult, Extractor, ExtractorType};
use aho_corasick::AhoCorasick;

// Defines the internal extractor function for carving out JPEG images
pub fn jpeg_extractor() -> Extractor {
    return Extractor {
        utility: ExtractorType::Internal(extract_jpeg_image),
        ..Default::default()
    };
}

pub fn extract_jpeg_image(
    file_data: &Vec<u8>,
    offset: usize,
    output_directory: Option<&String>,
) -> ExtractionResult {
    const OUTFILE_NAME: &str = "image.jpg";

    let mut result = ExtractionResult {
        ..Default::default()
    };

    // Find the JPEG EOF to identify the total JPEG size
    if let Some(jpeg_data_size) = get_jpeg_data_size(&file_data[offset..]) {
        result.size = Some(jpeg_data_size);
        result.success = true;

        if let Some(_) = output_directory {
            let chroot = Chroot::new(output_directory);
            result.success = chroot.carve_file(
                &OUTFILE_NAME.to_string(),
                file_data,
                offset,
                result.size.unwrap(),
            );
        }
    }

    return result;
}

fn get_jpeg_data_size(jpeg_data: &[u8]) -> Option<usize> {
    const EOF_SIZE: usize = 2;
    const JPEG_DELIM: u8 = 0xFF;

    // This is a short EOF marker to search for, but in a valid JPEG it *should* only occur at EOF
    let grep = AhoCorasick::new(vec![b"\xFF\xD9"]).unwrap();

    for eof_match in grep.find_overlapping_iter(jpeg_data) {
        let eof_candidate: usize = eof_match.start() + EOF_SIZE;

        // Make sure the expected EOF marker is not immediately followed by 0xFF (which would indicate the JPEG continues...)
        if eof_candidate < jpeg_data.len() {
            if jpeg_data[eof_candidate] == JPEG_DELIM {
                continue;
            }
        }

        return Some(eof_match.start() + EOF_SIZE);
    }

    return None;
}
