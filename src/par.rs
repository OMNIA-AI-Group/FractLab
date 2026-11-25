//! Calcul parallèle sur tous les cœurs du processeur (bibliothèque standard
//! seule) : les balayages de position du rotor sont indépendants les uns des
//! autres, chaque position est donc calculée par le premier fil disponible.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Nombre de fils de calcul (tous les cœurs logiques).
pub fn threads() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
}

/// `(0..n).map(f).collect()` réparti sur tous les cœurs, ordre conservé.
pub fn map<T: Send, F: Fn(usize) -> T + Sync>(n: usize, f: F) -> Vec<T> {
    let nt = threads().min(n.max(1));
    if nt <= 1 || n < 4 {
        return (0..n).map(f).collect();
    }
    let next = AtomicUsize::new(0);
    let mut parts: Vec<Vec<(usize, T)>> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..nt)
            .map(|_| {
                sc.spawn(|| {
                    let mut v = Vec::new();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        if i >= n {
                            break;
                        }
                        v.push((i, f(i)));
                    }
                    v
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("fil de calcul")).collect()
    });
    let mut out: Vec<Option<T>> = (0..n).map(|_| None).collect();
    for p in parts.iter_mut() {
        for (i, v) in p.drain(..) {
            out[i] = Some(v);
        }
    }
    out.into_iter().map(|v| v.expect("résultat manquant")).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn ordre_conserve() {
        let v = super::map(1000, |i| i * i);
        assert!(v.iter().enumerate().all(|(i, x)| *x == i * i));
        assert!(super::map(0, |i| i).is_empty());
    }
}
