use cronus::Cronus;

#[tokio::test]
async fn test_start_and_stop() {
    let cronus = Cronus::new();

    assert!(!cronus.is_running(), "Cronus shouldn't start by default");
    cronus.start();
    assert!(cronus.is_running(), "Cronus should start");
    cronus.stop();
    assert!(!cronus.is_running(), "Cronus should stop");
}
