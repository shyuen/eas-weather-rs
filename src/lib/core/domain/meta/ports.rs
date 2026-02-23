pub trait Meta: Clone + Send + Sync + 'static {
    /// Get application metadata, such as version info, uptime, etc.
    async fn get_app_data(&self) -> String;
}
