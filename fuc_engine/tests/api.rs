use std::fmt::Write;

use expect_test::expect_file;
use public_api::PublicApi;

#[test]
#[cfg_attr(miri, ignore)] // gnu_get_libc_version breaks miri
fn api() {
    let json_path = rustdoc_json::Builder::default()
        .all_features(true)
        .build()
        .unwrap();

    let mut golden = String::new();
    {
        let api = PublicApi::from_rustdoc_json(json_path, public_api::Options::default()).unwrap();
        for public_item in api.items() {
            writeln!(golden, "{public_item}").unwrap();
        }
    }
    expect_file!["../api.golden"].assert_eq(&golden);
}
