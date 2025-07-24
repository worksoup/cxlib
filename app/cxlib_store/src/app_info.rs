pub struct AppInfo {
    env_arg: &'static str,
    qualifier: &'static str,
    organization: &'static str,
    application: &'static str,
}
impl Default for AppInfo {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT_CONFIG_DIR_INFO
    }
}
impl AppInfo {
    const DEFAULT_CONFIG_DIR_INFO: AppInfo = AppInfo {
        env_arg: "TEST_CXSIGN",
        qualifier: "up.workso",
        organization: "Worksoup",
        application: "cxsign",
    };
    #[inline]
    pub fn new(
        env_arg: &'static str,
        qualifier: &'static str,
        organization: &'static str,
        application: &'static str,
    ) -> Self {
        Self {
            env_arg,
            qualifier,
            organization,
            application,
        }
    }
    #[inline]
    pub fn env_arg(&self) -> &'static str {
        self.env_arg
    }
    #[inline]
    pub fn qualifier(&self) -> &'static str {
        self.qualifier
    }
    #[inline]
    pub fn organization(&self) -> &'static str {
        self.organization
    }
    #[inline]
    pub fn application(&self) -> &'static str {
        self.application
    }
}
