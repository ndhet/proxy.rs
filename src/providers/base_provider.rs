use std::time::Duration;

use regex::Regex;
use reqwest::{Client, RequestBuilder};
use tokio::time::timeout;

use crate::{
    providers::{PROXIES, UNIQUE_PROXIES},
    utils::{http::random_useragent, run_parallel},
};

#[derive(Debug, Clone)]
pub struct BaseProvider {
    pub proto: Vec<String>,
    pub domain: String,
    pub client: Client,
    pub timeout: i32,
    pub max_tries: i32,
}

async fn get_html_with_timeout(task: RequestBuilder, timeout_in_sec: i32) -> Option<String> {
    if let Ok(fut) = timeout(Duration::from_secs(timeout_in_sec as u64), async {
        if let Ok(response) = task.send().await {
            if let Ok(body) = response.text().await {
                return body;
            }
        }
        String::new()
    })
    .await
    {
        return Some(fut);
    }
    None
}

impl BaseProvider {
    pub async fn get_html(&self, task: RequestBuilder) -> String {
        for _ in 0..self.max_tries {
            let task_c = task.try_clone().unwrap();
            if let Some(body) = get_html_with_timeout(task_c, self.timeout).await {
                return body;
            }
        }
        String::new()
    }

    pub async fn get_all_html(&self, tasks: Vec<RequestBuilder>) -> Vec<String> {
        let mut mapped_tasks = vec![];
        for task in tasks {
            let timeout = self.timeout;
            let fut = tokio::task::spawn(async move {
                if let Some(body) = get_html_with_timeout(task, timeout).await {
                    return body;
                }
                String::new()
            });
            mapped_tasks.push(fut);
        }
        let ret = run_parallel(mapped_tasks, None).await;
        ret.into_iter().map(|f| f.unwrap()).collect()
    }

    pub fn find_proxies(&self, pattern: String, html: &str) -> Vec<(String, u16, Vec<String>)> {
        let re = Regex::new(&pattern).unwrap();
        let mut proxies = vec![];
        for cap in re.captures_iter(html) {
            let ip = cap.get(1).unwrap().as_str();
            let port = cap.get(2).unwrap().as_str();

            if let Ok(port) = port.parse::<u16>() {
                proxies.push((ip.to_string(), port, self.proto.clone()))
            }
        }
        log::debug!("{} proxies received from {}", proxies.len(), self.domain);
        proxies
    }

    pub async fn update_stack(&self, proxies: &Vec<(String, u16, Vec<String>)>) {
        let mut added = 0;
        for (ip, port, proto) in proxies {
            //
            // if let Some(proxy) = Proxy::create(ip, *port, proto.to_vec()).await {
            //     let host_port = proxy.as_text();
            //
            //     let mut unique_proxy = UNIQUE_PROXIES.lock();
            //     if !unique_proxy.contains(&host_port) && PROXIES.push(proxy).is_ok() {
            //         added += 1;
            //         unique_proxy.push(host_port)
            //     }
            // }

            let host_port = format!("{}:{}", ip, port);
            let mut unique_proxy = UNIQUE_PROXIES.lock();
            if !unique_proxy.contains(&host_port)
                && PROXIES
                    .push((ip.to_owned(), *port, proto.to_owned()))
                    .is_ok()
            {
                added += 1;
                unique_proxy.push(host_port)
            }
        }

        log::debug!("{} proxies added(received) from {}", added, self.domain)
    }
}

impl Default for BaseProvider {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .user_agent(random_useragent(true))
                .build()
                .unwrap(),
            domain: String::new(),
            max_tries: 3,
            timeout: 8,
            proto: vec![],
        }
    }
}
