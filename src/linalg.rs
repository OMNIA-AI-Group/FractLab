//! Algèbre linéaire minimale : décomposition LU dense à pivot partiel.
//! Suffisant pour les systèmes du modèle en sous-domaines (quelques
//! centaines d'inconnues) ; la matrice est factorisée une fois puis
//! réutilisée pour chaque position du rotor.

#[derive(Clone)]
/// Factorisation LU dense (P·A = L·U) réutilisable pour plusieurs seconds membres.
pub struct Lu {
    n: usize,
    a: Vec<f64>, // facteurs L et U stockés en place, ligne par ligne
    piv: Vec<usize>,
}

impl Lu {
    /// Factorise la matrice carrée `m` (n×n, rangée par lignes).
    pub fn new(n: usize, mut a: Vec<f64>) -> Result<Lu, String> {
        assert_eq!(a.len(), n * n);
        let mut piv: Vec<usize> = (0..n).collect();
        for k in 0..n {
            // pivot
            let mut p = k;
            let mut best = a[k * n + k].abs();
            for i in k + 1..n {
                let v = a[i * n + k].abs();
                if v > best {
                    best = v;
                    p = i;
                }
            }
            if best == 0.0 || !best.is_finite() {
                return Err(format!("matrice singulière (colonne {k})"));
            }
            if p != k {
                for j in 0..n {
                    a.swap(k * n + j, p * n + j);
                }
                piv.swap(k, p);
            }
            let akk = a[k * n + k];
            // ligne k (pivot) et lignes suivantes sur des tranches contiguës :
            // la mise à jour est vectorisée par le compilateur
            let (top, rest) = a.split_at_mut((k + 1) * n);
            let pivot_row = &top[k * n + k + 1..k * n + n];
            for row in rest.chunks_exact_mut(n) {
                let f = row[k] / akk;
                row[k] = f;
                if f != 0.0 {
                    for (x, p) in row[k + 1..].iter_mut().zip(pivot_row) {
                        *x -= f * p;
                    }
                }
            }
        }
        Ok(Lu { n, a, piv })
    }

    /// Résout A·x = b avec la factorisation déjà calculée.
    pub fn solve(&self, b: &[f64]) -> Vec<f64> {
        let n = self.n;
        let mut x: Vec<f64> = self.piv.iter().map(|&p| b[p]).collect();
        for i in 0..n {
            let s = x[i] - dot(&self.a[i * n..i * n + i], &x[..i]);
            x[i] = s;
        }
        for i in (0..n).rev() {
            let s = x[i] - dot(&self.a[i * n + i + 1..i * n + n], &x[i + 1..]);
            x[i] = s / self.a[i * n + i];
        }
        x
    }
}

/// Produit scalaire à quatre accumulateurs indépendants (vectorisable).
fn dot(a: &[f64], b: &[f64]) -> f64 {
    let mut acc = [0.0f64; 4];
    let (ca, cb) = (a.chunks_exact(4), b.chunks_exact(4));
    let (ra, rb) = (ca.remainder(), cb.remainder());
    for (x, y) in ca.zip(cb) {
        acc[0] += x[0] * y[0];
        acc[1] += x[1] * y[1];
        acc[2] += x[2] * y[2];
        acc[3] += x[3] * y[3];
    }
    let mut s = (acc[0] + acc[1]) + (acc[2] + acc[3]);
    for (x, y) in ra.iter().zip(rb) {
        s += x * y;
    }
    s
}

/// Résout un petit système dense (copie la matrice).
pub fn solve_dense(n: usize, a: &[f64], b: &[f64]) -> Result<Vec<f64>, String> {
    Ok(Lu::new(n, a.to_vec())?.solve(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Grand système aléatoire bien conditionné : résidu au niveau de l'arrondi.
    #[test]
    fn lu_grand_systeme() {
        let n = 301;
        let mut seed = 7u64;
        let mut rnd = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 11) as f64 / (1u64 << 53) as f64 - 0.5
        };
        let mut a: Vec<f64> = (0..n * n).map(|_| rnd()).collect();
        for i in 0..n {
            a[i * n + i] += n as f64 * 0.3;
        }
        let b: Vec<f64> = (0..n).map(|_| rnd()).collect();
        let x = solve_dense(n, &a, &b).unwrap();
        let res = (0..n).map(|i| ((0..n).map(|j| a[i * n + j] * x[j]).sum::<f64>() - b[i]).abs()).fold(0.0, f64::max);
        assert!(res < 1e-11, "résidu {res}");
    }

    #[test]
    fn lu_3x3() {
        let a = vec![2.0, 1.0, -1.0, -3.0, -1.0, 2.0, -2.0, 1.0, 2.0];
        let x = solve_dense(3, &a, &[8.0, -11.0, -3.0]).unwrap();
        for (xi, e) in x.iter().zip([2.0, 3.0, -1.0]) {
            assert!((xi - e).abs() < 1e-12);
        }
    }
}
