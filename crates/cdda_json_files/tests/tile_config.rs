#![cfg(test)]

use cdda_json_files::CddaTileConfig;
use reqwest::blocking::get as get_request;
use std::fs::read;

fn load(bytes: &[u8]) {
    let result = serde_json::from_slice::<CddaTileConfig>(bytes);
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn from_github() {
    let github_url = "https://raw.githubusercontent.com/CleverRaven/Cataclysm-DDA/293c7615db5b1988916cc3120cb1cf398246b92b/gfx/UltimateCataclysm/tile_config.json";
    let github_response_body = get_request(github_url)
        .expect("Github request should work")
        .bytes()
        .expect("Github response should be complete");
    load(&github_response_body);
}

#[test]
fn local_file_if_exists() {
    let file_path = "../../assets/gfx/UltimateCataclysm/tile_config.json";
    // Uncomment to check the path when the file is in place:
    //read(file_path).expect("The path should be valid");

    if let Ok(file_contents) = read(file_path) {
        load(&file_contents);
    }
}
