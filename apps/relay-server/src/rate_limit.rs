//! Rate limiting par IP pour CreateSession / JoinSession.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Limiteur à fenêtre glissante (60 s) indexé par adresse IP.
#[derive(Debug)]
pub struct RateLimiter {
    limit: u32,
    window: Duration,
    inner: Mutex<HashMap<IpAddr, Vec<Instant>>>,
}

impl RateLimiter {
    /// Crée un limiteur autorisant `limit_per_min` tentatives par IP et par minute.
    ///
    /// `limit_per_min == 0` désactive le limiteur (toujours autorisé).
    pub fn new(limit_per_min: u32) -> Self {
        Self {
            limit: limit_per_min,
            window: Duration::from_secs(60),
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Enregistre une tentative et retourne `true` si elle est autorisée.
    pub fn check_and_record(&self, ip: IpAddr) -> bool {
        if self.limit == 0 {
            return true;
        }
        let now = Instant::now();
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let entries = map.entry(ip).or_default();
        entries.retain(|t| now.duration_since(*t) < self.window);
        if (entries.len() as u32) >= self.limit {
            return false;
        }
        entries.push(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn allows_under_limit() {
        let lim = RateLimiter::new(3);
        let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        assert!(lim.check_and_record(ip));
        assert!(lim.check_and_record(ip));
        assert!(lim.check_and_record(ip));
        assert!(!lim.check_and_record(ip));
    }

    #[test]
    fn disabled_when_zero() {
        let lim = RateLimiter::new(0);
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        for _ in 0..100 {
            assert!(lim.check_and_record(ip));
        }
    }

    #[test]
    fn isolates_ips() {
        let lim = RateLimiter::new(1);
        let a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
        assert!(lim.check_and_record(a));
        assert!(!lim.check_and_record(a));
        assert!(lim.check_and_record(b));
    }
}
