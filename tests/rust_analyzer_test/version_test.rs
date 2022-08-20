use rust_analyzer_downloader::rust_analyzer::version::get;
use tokio::process::Command;

#[tokio::test]
async fn test_get_command_not_found() {
    let real_version = Command::new("./rust-analyzer")
        .arg("--version")
        .output()
        .await;

    assert!(real_version.is_err());
}

#[tokio::test]
async fn test_get_success() {
    let real_version = Command::new("rust-analyzer")
        .arg("--version")
        .output()
        .await
        .unwrap()
        .stdout;

    let real_version = String::from_utf8(real_version).unwrap();
    let real_version = real_version.split(' ').collect::<Vec<&str>>();

    let version = get().await;

    assert!(version.is_ok());

    let version = version.unwrap();
    assert_eq!(
        version.semantic_version,
        real_version.get(1).unwrap().to_owned()
    );

    assert_eq!(
        version.date_version,
        real_version
            .get(3)
            .unwrap()
            .to_owned()
            .strip_suffix(")\n")
            .unwrap()
            .to_owned()
    );
}
