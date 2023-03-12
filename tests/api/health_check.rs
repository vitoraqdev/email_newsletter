use crate::helpers::spawn_app;

#[actix_web::test]
async fn health_check_works() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    let response = reqwest::get(format!("{}/health_check", test_app.address))
        .await
        .unwrap();

    // Assert
    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}
