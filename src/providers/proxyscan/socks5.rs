use crate::{providers::base_provider::BaseProvider, utils::vec_of_strings};

#[derive(Debug, Clone)]
pub struct ProxyscanIoSocks5Provider {
    pub base: BaseProvider,
    pub url: String,
    pub pattern: String,
}

impl ProxyscanIoSocks5Provider {
    pub async fn get_proxies(&mut self) -> Vec<(String, u16, Vec<String>)> {
        let req = self.base.client.get(self.url.clone());
        let html = self.base.get_html(req).await;
        let proxies = self.base.find_proxies(self.pattern.clone(), html.as_str());
        self.base.update_stack(&proxies).await;

        proxies
    }
}

impl Default for ProxyscanIoSocks5Provider {
    fn default() -> Self {
        Self {
            base: BaseProvider {
                proto: vec_of_strings!["SOCKS5"],
                domain: "proxyscan.io/socks5".to_string(),
                ..Default::default()
            },
            url: "https://www.proxyscan.io/download?type=socks5".to_string(),
            pattern: r#"(?P<ip>(?:\d+\.?){4})\:(?P<port>\d+)"#.to_string(),
        }
    }
}
